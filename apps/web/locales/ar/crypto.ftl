# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: crypto. Source: ../en/crypto.ftl

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = مشفّر بين الطرفين
crypto-banner-e2ee-detail = لكل مستلم مفتاح — يُشفَّر نص الرسالة على هذا الجهاز.
crypto-banner-tls-label = تشفير النقل (TLS)
crypto-banner-tls-detail = لم يُعثر على مفاتيح تشفير للمستلمين؛ التسليم محمي أثناء النقل فقط.
crypto-banner-mixed-label = حماية مختلطة
crypto-banner-mixed-detail = يمكن لبعض المستلمين تلقي التشفير بين الطرفين؛ ويحصل الآخرون على TLS فقط.
crypto-banner-checking-label = جارٍ التحقق من مفاتيح المستلمين
crypto-banner-checking = · جارٍ التحقق…
crypto-banner-tls-only = TLS فقط لـ:

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = أمان الرسالة
crypto-encrypt-label = تشفير (بين الطرفين)
crypto-encrypt-hint-default = تشفير النص على هذا الجهاز قبل الإرسال.
crypto-encrypt-drafted = شُفّرت المسودة على هذا الجهاز.
crypto-encrypt-no-key = لا يتوفر مفتاح تشفير للمستلم.
crypto-sign-label = توقيع (للتحقق من أنها منك)
crypto-protect-subject = تشفير سطر الموضوع أيضًا
crypto-reason-add-recipient = أضف مستلمًا للتحقق من مفاتيح التشفير.
crypto-reason-tls = لا يتوفر مفتاح تشفير للمستلم — يجري الإرسال عبر TLS.

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = تحذيرات منع فقدان البيانات
crypto-dlp-block = الإرسال محظور
crypto-dlp-require = التشفير مطلوب
crypto-dlp-warn = تنبيه
crypto-dlp-notify = سيُبلَّغ المسؤول
crypto-dlp-matched = تطابق: { $list }
