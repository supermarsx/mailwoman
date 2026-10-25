# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: keys. Source: ../en/keys.ftl

# -- Panel header ------------------------------------------------------------
keys-panel-label = إدارة المفاتيح
keys-title = المفاتيح والشهادات
keys-subtitle = مفاتيح OpenPGP وS/MIME. تبقى المفاتيح الخاصة على هذا الجهاز ولا تصل إلى الخادم أبدًا.
keys-generate = توليد مفتاح
keys-import = استيراد مفتاح

# -- Key list groups ---------------------------------------------------------
keys-your-keys = مفاتيحك
keys-empty-own = لا توجد مفاتيح بعد. ولّد مفتاحًا أو استورده للبدء.
keys-contact-keys = مفاتيح جهات الاتصال
keys-empty-contact = لا توجد مفاتيح لجهات الاتصال. ابحث عن مفتاح أدناه.
keys-loading = جارٍ تحميل المفاتيح…
keys-select-prompt = اختر مفتاحًا لعرضه والتحقق منه.
keys-untitled = مفتاح { $kind }

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = غير متحقَّق منه
keys-trust-tofu = ثقة عند أول استخدام (TOFU)
keys-trust-verified = متحقَّق منه
keys-trust-revoked = ملغى

# -- Key detail --------------------------------------------------------------
keys-key-card = المفتاح { $name }
keys-fingerprint = البصمة
keys-safe-words = كلمات الأمان
keys-safe-words-help = اقرأها بصوت عالٍ مع جهة الاتصال لتأكيد المفتاح عبر قناة مستقلة.
keys-safe-words-list = كلمات أمان البصمة
keys-scan-label = امسح للتحقق
keys-qr-label = رمز QR للبصمة
keys-autocrypt = Autocrypt
keys-autocrypt-on = مُعلَن في ترويسات Autocrypt
keys-autocrypt-off = غير مُعلَن عبر Autocrypt
keys-trust-label = الثقة
keys-trust-level = مستوى الثقة
keys-associate-label = الربط بجهة اتصال
keys-associate-select = جهة الاتصال المراد ربطها
keys-choose-contact = اختر جهة اتصال…
keys-associate-button = ربط
keys-backup-label = النسخ الاحتياطي
keys-backup-help = صدّر رسالة إعداد Autocrypt لنقل هذا المفتاح إلى جهاز آخر.
keys-export-backup = تصدير نسخة احتياطية
keys-asm-label = رسالة إعداد Autocrypt
keys-no-private-backup = لا يوجد مفتاح خاص على هذا الجهاز للنسخ الاحتياطي

# -- Own S/MIME key: certificate, export, request, issued certificate ---------
keys-smime-section = شهادة S/MIME
keys-smime-self-signed = هذه الشهادة موقّعة ذاتيًا: وُقّعت بمفتاحها نفسه، لا من جهة إصدار شهادات. لن تثق بها برامج البريد لدى الآخرين ما لم يقبلوها يدويًا، أو حتى تُصدر جهة إصدار شهادات شهادةً من ملف الطلب أدناه.
keys-smime-valid-until = صالحة حتى { $date }.
keys-smime-not-in-app = لا يوقّع Mailwoman البريد ولا يفك تشفيره بمفتاح S/MIME حتى الآن. صدّر المفتاح لاستخدامه في برنامج بريد آخر.
keys-smime-passphrase = عبارة المرور لمفتاح S/MIME هذا
keys-smime-export-help = يحتوي ملف .p12 على الشهادة والمفتاح الخاص. يُشفَّر المفتاح بعبارة المرور أعلاه، والعبارة نفسها تفتح الملف في برنامج آخر. لا يحتوي ملف .csr على أي مفتاح خاص.
keys-smime-export = تصدير بصيغة PKCS#12 (.p12)
keys-smime-request = تنزيل طلب الشهادة (.csr)
keys-smime-issued-label = الشهادة الصادرة لهذا المفتاح (.pem، .crt، .cer)
keys-smime-issued-file = ملف الشهادة الصادرة
keys-smime-issued-help = تحلّ محل الشهادة المحفوظة لهذا المفتاح. تُرفض الشهادة الصادرة لمفتاح مختلف.
keys-smime-attach = استيراد الشهادة الصادرة
keys-smime-failed = لم ينجح ذلك. تحقق من عبارة المرور.
keys-smime-attach-failed = لم تُستورد الشهادة. تحقق من عبارة المرور، ومن أن الملف شهادة واحدة صادرة لهذا المفتاح.

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = البحث عن مفتاح جهة اتصال
keys-lookup-heading = البحث عن مفتاح
keys-lookup-address = العنوان المراد البحث عنه
keys-lookup-placeholder = name@example.org
keys-lookup-sources = مصادر البحث
keys-source = المصدر { $source }
keys-consent-label = الموافقة على البحث الخارجي
keys-consent-text = البحث عن مفتاح يتصل بأدلة خارجية (WKD/VKS). لديّ موافقة هذا الشخص على ذلك.
keys-lookup-button = بحث
keys-looking-up = جارٍ البحث…
keys-lookup-found = { $count ->
    [zero] لم يُعثر على أي مفتاح.
    [one] عُثر على مفتاح واحد — أُضيف إلى مفاتيح جهات الاتصال.
    [two] عُثر على مفتاحين — أُضيفا إلى مفاتيح جهات الاتصال.
    [few] عُثر على { $count } مفاتيح — أُضيفت إلى مفاتيح جهات الاتصال.
    [many] عُثر على { $count } مفتاحًا — أُضيفت إلى مفاتيح جهات الاتصال.
   *[other] عُثر على { $count } مفتاح — أُضيفت إلى مفاتيح جهات الاتصال.
}
keys-lookup-none = لم يُعثر على أي مفتاح.

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = توليد مفتاح
keys-type = النوع
keys-key-type = نوع المفتاح
keys-openpgp = OpenPGP
keys-smime = S/MIME
keys-generate-smime-note = يُنشئ مفتاح RSA بطول 3072 بت وشهادة للعنوان أدناه، صالحة لمدة سنتين. الشهادة موقّعة ذاتيًا: لن تثق بها برامج البريد لدى الآخرين ما لم يقبلوها يدويًا، أو حتى تُصدر جهة إصدار شهادات شهادةً لها. لا يوقّع Mailwoman البريد ولا يفك تشفيره بها حتى الآن؛ ويمكنك لاحقًا تصديرها لبرنامج بريد آخر.
keys-name = الاسم
keys-email = البريد الإلكتروني
keys-key-passphrase = عبارة مرور المفتاح
keys-passphrase-help = تحمي عبارة المرور المفتاح الخاص على هذا الجهاز. ولا تغادر المتصفح أبدًا.
keys-cancel = إلغاء
keys-generate-submit = توليد
keys-generating = جارٍ التوليد…
keys-generating-smime = جارٍ توليد مفتاح RSA في هذا المتصفح. قد يستغرق ذلك حتى دقيقة.
keys-generate-failed = لم يُولَّد أي مفتاح.
keys-generate-failed-smime = لم يُولَّد أي مفتاح. لا يمكن للشهادة أن تحمل إلا عنوان ASCII عاديًا (name@domain)، واسمًا لا يتجاوز 64 حرفًا.

# -- Import dialog -----------------------------------------------------------
keys-import-title = استيراد مفتاح
keys-import-type = نوع الاستيراد
keys-tab-armored = نصي مُدرَّع (PGP)
keys-tab-pkcs12 = PKCS#12 (S/MIME)
keys-armored-key = مفتاح نصي مُدرَّع
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = عبارة المرور (إذا كان المفتاح خاصًا)
keys-import-passphrase = عبارة مرور الاستيراد
keys-pkcs12-file-label = ملف PKCS#12 (.p12 / .pfx)
keys-pkcs12-file = ملف PKCS#12
keys-import-password-label = كلمة مرور الاستيراد
keys-pkcs12-password = كلمة مرور PKCS#12
keys-import-preview = معاينة الاستيراد
keys-preview = معاينة
keys-preview-type = النوع: { $kind }
keys-preview-fingerprint-aria = بصمة المعاينة
keys-preview-fingerprint = البصمة: { $fp }
keys-preview-has-private = يتضمن مفتاحًا خاصًا (سيُخزَّن على هذا الجهاز)
keys-preview-public-only = مفتاح عام فقط
keys-import-submit = استيراد
