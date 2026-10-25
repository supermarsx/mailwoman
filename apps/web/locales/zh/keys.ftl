# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: keys. Source: ../en/keys.ftl

# -- Panel header ------------------------------------------------------------
keys-panel-label = 密钥管理
keys-title = 密钥和证书
keys-subtitle = OpenPGP 和 S/MIME 密钥。私钥保留在此设备上，永远不会发送到服务器。
keys-generate = 生成密钥
keys-import = 导入密钥

# -- Key list groups ---------------------------------------------------------
keys-your-keys = 你的密钥
keys-empty-own = 还没有密钥。生成或导入一个密钥即可开始。
keys-contact-keys = 联系人密钥
keys-empty-contact = 没有联系人密钥。可在下方查询。
keys-loading = 正在加载密钥…
keys-select-prompt = 选择一个密钥以查看和验证。
keys-untitled = { $kind } 密钥

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = 未验证
keys-trust-tofu = 首次使用时信任
keys-trust-verified = 已验证
keys-trust-revoked = 已吊销

# -- Key detail --------------------------------------------------------------
keys-key-card = 密钥 { $name }
keys-fingerprint = 指纹
keys-safe-words = 安全词
keys-safe-words-help = 与联系人一起大声读出这些词，通过其他渠道确认密钥。
keys-safe-words-list = 指纹安全词
keys-scan-label = 扫描以验证
keys-qr-label = 指纹二维码
keys-autocrypt = Autocrypt
keys-autocrypt-on = 在 Autocrypt 标头中公布
keys-autocrypt-off = 未通过 Autocrypt 公布
keys-trust-label = 信任
keys-trust-level = 信任级别
keys-associate-label = 关联到联系人
keys-associate-select = 要关联的联系人
keys-choose-contact = 选择联系人…
keys-associate-button = 关联
keys-backup-label = 备份
keys-backup-help = 导出 Autocrypt 设置邮件，以便将此密钥转移到另一台设备。
keys-export-backup = 导出备份
keys-asm-label = Autocrypt 设置邮件
keys-no-private-backup = 此设备上没有可备份的私钥

# -- Own S/MIME key: certificate, export, request, issued certificate ---------
keys-smime-section = S/MIME 证书
# Shown for a key generated here (not for an imported or CA-issued certificate).
keys-smime-self-signed = 此证书是自签名证书：它由自身的密钥签名，而不是由证书颁发机构签名。除非对方手动接受，或者证书颁发机构根据下方的请求文件颁发了证书，否则其他人的邮件程序不会信任它。
# $date is YYYY-MM-DD, taken from the certificate.
keys-smime-valid-until = 有效期至 { $date }。
keys-smime-not-in-app = Mailwoman 目前还不能使用 S/MIME 密钥对邮件签名或解密。请导出密钥，在其他邮件程序中使用。
keys-smime-passphrase = 此 S/MIME 密钥的密码短语
keys-smime-export-help = .p12 文件包含证书和私钥。私钥使用上方的密码短语加密，在其他程序中打开该文件也使用同一密码短语。.csr 文件不包含私钥。
keys-smime-export = 导出为 PKCS#12 (.p12)
keys-smime-request = 下载证书请求 (.csr)
keys-smime-issued-label = 为此密钥颁发的证书（.pem、.crt、.cer）
keys-smime-issued-file = 已颁发的证书文件
keys-smime-issued-help = 替换此密钥当前保存的证书。为其他密钥制作的证书会被拒绝。
keys-smime-attach = 导入已颁发的证书
keys-smime-failed = 操作未成功。请检查密码短语。
keys-smime-attach-failed = 证书未导入。请检查密码短语，并确认该文件是为此密钥颁发的单个证书。

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = 查询联系人密钥
keys-lookup-heading = 查询密钥
keys-lookup-address = 要查询的地址
keys-lookup-placeholder = name@example.org
keys-lookup-sources = 查询来源
keys-source = 来源 { $source }
keys-consent-label = 同意外部查询
keys-consent-text = 查询密钥会联系外部目录（WKD/VKS）。我已获得此人的同意。
keys-lookup-button = 查询
keys-looking-up = 正在查询…
keys-lookup-found = 找到 { $count } 个密钥——已添加到联系人密钥。
keys-lookup-none = 未找到密钥。

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = 生成密钥
keys-type = 类型
keys-key-type = 密钥类型
keys-openpgp = OpenPGP
keys-smime = S/MIME
# Under the type list while S/MIME is chosen: what will be made, and its limits.
keys-generate-smime-note = 为下方的地址生成一个 RSA 3072 位密钥和一个证书，有效期为两年。该证书是自签名的：除非对方手动接受，或者证书颁发机构颁发了证书，否则其他人的邮件程序不会信任它。Mailwoman 目前还不能用它对邮件签名或解密；生成后可以将其导出，供其他邮件程序使用。
keys-name = 姓名
keys-email = 电子邮件
keys-key-passphrase = 密钥密码短语
keys-passphrase-help = 密码短语用于在此设备上封装私钥。它永远不会离开浏览器。
keys-cancel = 取消
keys-generate-submit = 生成
keys-generating = 正在生成…
keys-generating-smime = 正在此浏览器中生成 RSA 密钥。这可能需要最多一分钟。
keys-generate-failed = 未生成密钥。
keys-generate-failed-smime = 未生成密钥。证书只能包含纯 ASCII 地址（name@domain），且名称最多 64 个字符。

# -- Import dialog -----------------------------------------------------------
keys-import-title = 导入密钥
keys-import-type = 导入类型
keys-tab-armored = ASCII 封装（PGP）
keys-tab-pkcs12 = PKCS#12（S/MIME）
keys-armored-key = ASCII 封装的密钥
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = 密码短语（如果是私钥）
keys-import-passphrase = 导入密码短语
keys-pkcs12-file-label = PKCS#12 文件（.p12 / .pfx）
keys-pkcs12-file = PKCS#12 文件
keys-import-password-label = 导入密码
keys-pkcs12-password = PKCS#12 密码
keys-import-preview = 导入预览
keys-preview = 预览
keys-preview-type = 类型：{ $kind }
keys-preview-fingerprint-aria = 预览指纹
keys-preview-fingerprint = 指纹：{ $fp }
keys-preview-has-private = 包含私钥（将保存在此设备上）
keys-preview-public-only = 仅公钥
keys-import-submit = 导入
