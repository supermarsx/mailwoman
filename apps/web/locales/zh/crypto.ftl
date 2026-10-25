# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: crypto. Source: ../en/crypto.ftl

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = 端到端加密
crypto-banner-e2ee-detail = 每个收件人都有密钥——邮件正文在此设备上加密。
crypto-banner-tls-label = 传输加密（TLS）
crypto-banner-tls-detail = 未找到收件人的加密密钥；仅在传输过程中受到保护。
crypto-banner-mixed-label = 混合保护
crypto-banner-mixed-detail = 部分收件人可接收端到端加密邮件；其他收件人仅有 TLS 保护。
crypto-banner-checking-label = 正在检查收件人密钥
crypto-banner-checking = · 正在检查…
crypto-banner-tls-only = 仅 TLS：

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = 邮件安全
crypto-encrypt-label = 加密（端到端）
crypto-encrypt-hint-default = 发送前在此设备上加密正文。
crypto-encrypt-drafted = 草稿已在此设备上加密。
crypto-encrypt-no-key = 没有可用的收件人加密密钥。
crypto-sign-label = 签名（证明邮件由你发出）
crypto-protect-subject = 同时加密主题行
crypto-reason-add-recipient = 添加收件人以检查加密密钥。
crypto-reason-tls = 没有可用的收件人加密密钥——将通过 TLS 发送。

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = 数据防泄漏警告
crypto-dlp-block = 已阻止发送
crypto-dlp-require = 需要加密
crypto-dlp-warn = 请注意
crypto-dlp-notify = 将通知管理员
crypto-dlp-matched = 匹配项：{ $list }
