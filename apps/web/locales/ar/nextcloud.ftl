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
