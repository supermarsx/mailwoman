# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = للأعلى
nextcloud-loading = جارٍ التحميل…
nextcloud-list-error = تعذّر سرد محتويات هذا المجلد.
nextcloud-empty = هذا المجلد فارغ.
nextcloud-file-list = ملفات Nextcloud
nextcloud-open-folder = فتح المجلد { $name }
nextcloud-select-file = تحديد { $name }

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = إرفاق من Nextcloud
nextcloud-attach-action = { $count ->
    [0] إرفاق
    [zero] إرفاق
    [one] إرفاق ملف واحد
    [two] إرفاق ملفين
    [few] إرفاق { $count } ملفات
    [many] إرفاق { $count } ملفًا
   *[other] إرفاق { $count } ملف
  }
nextcloud-error-select-file = حدّد ملفًا واحدًا على الأقل لإرفاقه
nextcloud-error-attach-failed = تعذّر إرفاق الملفات المحددة

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = حفظ في Nextcloud
nextcloud-save-title = حفظ «{ $name }» في Nextcloud
nextcloud-save-action = الحفظ هنا ({ $dir })
nextcloud-saved = حُفظ في { $path }
nextcloud-error-save-failed = تعذّر الحفظ في Nextcloud

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = إنشاء رابط مشاركة
nextcloud-share-title = رابط المشاركة
nextcloud-share-intro = أنشئ رابطًا عامًا إلى { $path } بدلًا من إرفاق الملف.
nextcloud-protect-password = الحماية بكلمة مرور
nextcloud-password-label = كلمة المرور
nextcloud-share-password = كلمة مرور المشاركة
nextcloud-set-expiry = تعيين تاريخ انتهاء
nextcloud-expires-on = ينتهي في
nextcloud-expiry-date = تاريخ الانتهاء
nextcloud-create-link = إنشاء الرابط
nextcloud-password-protected = محمي بكلمة مرور
nextcloud-no-password = بلا كلمة مرور
nextcloud-expires = ينتهي في { $date }
nextcloud-no-expiry = بلا تاريخ انتهاء
nextcloud-error-need-password = أدخل كلمة مرور أو أوقف الحماية بكلمة المرور
nextcloud-error-need-expiry = اختر تاريخ انتهاء أو أوقف الانتهاء
nextcloud-error-share-failed = تعذّر إنشاء رابط المشاركة

# -- Machine translation (2026-11): added to match en ---------------------
nextcloud-share-upload-intro = { $path } أكبر من حد المرفقات. ارفعه إلى مجلد «Mail attachments» في Nextcloud الخاص بك وأدرج رابط مشاركة في الرسالة بدلًا منه.
nextcloud-upload-and-link = رفع وإدراج الرابط
nextcloud-share-not-now = ليس الآن
nextcloud-upload-progress = جارٍ الرفع: { $percent }%
nextcloud-upload-progress-label = تقدم الرفع
nextcloud-share-inserted = تم إدراج رابط مشاركة { $name } في الرسالة.
nextcloud-share-body = { $name } ({ $size }): { $url }
nextcloud-share-body-expires = يعمل الرابط حتى { $date }.
nextcloud-share-body-password = الرابط محمي بكلمة مرور.
nextcloud-account-title = حسابات Nextcloud
nextcloud-account-help = اربط حسابك الخاص بمثيل وافق عليه المسؤول. تُخزَّن بيانات الاعتماد مشفّرة لحساب البريد هذا فقط.
nextcloud-account-load-error = تعذّر تحميل روابط Nextcloud. أعد التحميل للمحاولة مرة أخرى.
nextcloud-account-unlink-error = تعذّرت إزالة الرابط. أعد التحميل قبل المحاولة مرة أخرى.
nextcloud-account-reload = إعادة التحميل
nextcloud-account-no-instances = لا توجد مثيلات Nextcloud مفعّلة متاحة. اطلب من المسؤول ضبط واحد.
nextcloud-account-ready = مرتبط وجاهز
nextcloud-account-stale = تغيّرت إعدادات المثيل. تحقق من الرابط مرة أخرى.
nextcloud-account-unlinked = غير مرتبط
nextcloud-account-relink = التحقق من الرابط مرة أخرى
nextcloud-account-link = ربط الحساب
nextcloud-account-unlink = إلغاء ربط الحساب
nextcloud-account-password-help = أنشئ كلمة مرور تطبيق من إعدادات الأمان الشخصية في Nextcloud وانسخ اسم المستخدم وكلمة المرور هنا. يرسل التحقق طلبًا واحدًا محدودًا؛ ولا تُعاد محاولة بيانات الاعتماد المرفوضة.
nextcloud-account-username = اسم مستخدم كلمة مرور تطبيق Nextcloud
nextcloud-account-password = كلمة مرور تطبيق Nextcloud
nextcloud-account-verify = تحقق واربط
nextcloud-account-verify-error = فشل التحقق أو الحفظ. تحقق من المثيل واسم مستخدم كلمة مرور التطبيق وكلمة المرور. تم مسح حقل كلمة المرور؛ ولم يُستبدل رابط عامل موجود.
nextcloud-admin-title = مثيلات Nextcloud
nextcloud-admin-help = وافق على مثيلات HTTPS التي يمكن لمستخدمي البريد ربطها فرديًا. لا تُمنح حسابات البريد كلمة مرور خدمة مشتركة. تقتصر عمليات الملفات على 8 ميبيبايت. يكتشف مستخدمو المحرك الأصلي مجموعات التقويم والمهام ودفتر العناوين ويربطونها صراحةً بعد الربط.
nextcloud-admin-add = إضافة مثيل Nextcloud
nextcloud-admin-edit = تعديل مثيل Nextcloud
nextcloud-admin-enabled = مفعّل
nextcloud-admin-disabled = معطّل
nextcloud-admin-missing = لم يعد هذا المثيل موجودًا. ارجع إلى قائمة المثيلات.
nextcloud-admin-private-help = تتطلب العناوين الخاصة إذنًا صريحًا بالاتصال الصادر إلى Nextcloud. تبقى مسارات الوكيل المضبوطة هي المرجع؛ ولا يعود الطلب الذي رفضه الوكيل إلى اتصال مباشر أبدًا.
nextcloud-admin-revalidation = يتطلب حفظ أي تغيير في المثيل أن يتحقق المستخدمون من روابطهم مرة أخرى. يوقف تعطيل المثيل العمليات الجديدة فورًا.
nextcloud-admin-authority = صلاحية المثيل
nextcloud-admin-name = الاسم المعروض
nextcloud-admin-name-help = اسم قصير يظهر للمستخدمين عند اختيار مثيل.
nextcloud-admin-url = عنوان URL لتثبيت HTTPS
nextcloud-admin-url-help = استخدم جذر التثبيت، مثل https://cloud.example.org/ أو https://example.org/nextcloud/. يجب أن تكون الشهادة موثوقة ومطابقة لاسم المضيف. تُرفض بيانات الاعتماد ومعلمات الاستعلام والأجزاء.
nextcloud-admin-enable = السماح للمستخدمين بربط هذا المثيل
nextcloud-admin-ca = جهات إصدار شهادات إضافية (PEM)
nextcloud-admin-ca-help = شهادات CA خاصة اختيارية لخادم Nextcloud هذا. تبقى الثقة العامة متاحة ويظل التحقق من اسم المضيف مطلوبًا. تتطلب التغييرات أن يتحقق المستخدمون من روابطهم مرة أخرى؛ والثقة بشهادة الوكيل منفصلة.
nextcloud-instance-select = مثيل Nextcloud
nextcloud-instance-placeholder = اختر مثيلًا مرتبطًا
nextcloud-instances-empty = لا توجد روابط Nextcloud تم التحقق منها. أضف رابطًا أو تحقق منه في إعدادات المستخدم ← Nextcloud.
nextcloud-instances-error = تعذّر تحميل المثيلات المرتبطة. أعد فتح أداة الاختيار للمحاولة مرة أخرى.
nextcloud-save-warning = يشترط المسؤول التأكيد قبل أن يغادر هذا المرفق صندوق البريد.
nextcloud-save-acknowledge = أفهم تحذير المرفق وأريد حفظ هذا الملف في مثيل Nextcloud المحدد.
nextcloud-save-attachment = حفظ { $name } في Nextcloud
nextcloud-save-close = إغلاق وجهة Nextcloud
nextcloud-save-options = حفظ المرفقات في Nextcloud
nextcloud-pim-title = جهات الاتصال والتقاويم
nextcloud-pim-help = اكتشف المجموعات من حساب Nextcloud هذا الذي تم التحقق منه، ثم اربط صراحةً ما تريده منها. زامِن لجلب البيانات الحالية؛ وتستخدم التعديلات على المجموعات المرتبطة المجموعة البعيدة وتفشل إذا أُلغي رابطها أو أذوناتها.
nextcloud-pim-category = فئة المجموعة
nextcloud-pim-calendar = التقاويم
nextcloud-pim-contacts = دفاتر العناوين
nextcloud-pim-tasks = قوائم المهام
nextcloud-pim-discover = اكتشاف المجموعات
nextcloud-pim-discovered = المجموعات المتاحة
nextcloud-pim-empty = لم تُرجَع أي مجموعات مطابقة.
nextcloud-pim-connect = ربط
nextcloud-pim-connected = المجموعات المرتبطة
nextcloud-pim-sync = مزامنة الآن
nextcloud-pim-sync-needed = تم ربط المجموعة. زامِن الآن لتحميل محتوياتها.
nextcloud-pim-synced = اكتملت المزامنة.
nextcloud-pim-error = فشلت العملية. تحقق من رابط الحساب والأذونات، ثم أعد المحاولة صراحةً. لم تُهمَل البيانات المخزنة مؤقتًا.
nextcloud-pim-native-required = تتطلب مزامنة جهات الاتصال والتقويم محرك البريد الأصلي. يظل الوصول إلى الملفات متاحًا عبر رابط حسابك.
nextcloud-pim-disconnect = فصل
nextcloud-pim-disconnect-help = يحافظ الفصل على البيانات المخزنة مؤقتًا ويوقف الكتابة البعيدة. تتطلب الحسابات المعاد ربطها الاكتشاف والربط الصريح مرة أخرى.
nextcloud-pim-ready = جاهز
nextcloud-pim-disconnected = مفصول أو تغيّر الرابط
nextcloud-pim-unnamed = مجموعة بلا اسم
nextcloud-pim-back = العودة إلى حسابات Nextcloud
nextcloud-pim-create-title = مجموعة جديدة في Nextcloud
nextcloud-pim-create-help = ينشئ تقويمًا أو قائمة مهام فارغة في حساب Nextcloud هذا ويربطها هنا.
nextcloud-pim-create-name = اسم المجموعة الجديدة
nextcloud-pim-create = إنشاء في Nextcloud
nextcloud-pim-created = تم إنشاء { $name } في Nextcloud وربطها. زامِن الآن لتحميلها.
nextcloud-pim-interval = فاصل التحديث التلقائي (بالثواني)
nextcloud-pim-interval-help = الصفر يعني التحديث اليدوي فقط. للفواصل الموجبة حد أدنى 60 ثانية. تُحدَّث الحسابات المرتبطة في الخلفية؛ وتتباطأ المحاولات عند الفشل، وتوقف بيانات الاعتماد المرفوضة الطلبات اللاحقة حتى تعيد الربط.
nextcloud-pim-interval-reset = استخدام الإعداد الافتراضي للمسؤول
nextcloud-admin-empty = لا توجد مثيلات معتمدة بعد. أضف مثيلًا ليصبح متاحًا لربط الحسابات.
nextcloud-admin-status = الحالة
nextcloud-admin-open = فتح { $name }
