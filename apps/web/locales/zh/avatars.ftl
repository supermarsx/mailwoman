# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: avatars. Source: ../en/avatars.ftl


# -- Machine translation (2026-11): added to match en ---------------------
avatars-title = 联系人和头像
avatars-mode = 发件人和联系人头像
avatars-off = 隐藏头像
avatars-initials = 在本地生成首字母头像
avatars-contacts = 使用本地联系人头像，无头像时使用首字母
avatars-help = 首字母头像在此设备上生成。联系人头像仅使用已保存在你自己通讯录中的内嵌 PNG 或 JPEG 照片，最大 512 × 512 像素、128 KiB。绝不会请求远程头像 URL 或外部头像服务。此偏好设置保存在你的账户中，管理员可以锁定。
avatars-default = 恢复继承的偏好设置
avatars-sender-icons = 显示内置的发件域名图标
avatars-sender-icons-help = 已识别的社交、开发者、AI、购物和政府域名会使用此应用内置的图标。你自己的联系人头像优先。未知发件人使用在本地生成的 192 种颜色的首字母头像。图标只是域名提示，并不证明邮件真实可信。不会联系任何外部徽标服务。
avatars-icons-default = 恢复继承的图标偏好设置
avatars-brand-avatars = 在已验证的发件人旁显示品牌徽标
avatars-brand-avatars-help = 在阅读器中，只有当邮件通过了 DKIM 以及与该公司自有域名对齐的 DMARC 检查时，已识别的公司才会在发件人旁显示其徽标。否则使用常规头像。徽标内置于此应用；不会向发件人请求任何内容。
avatars-brand-avatars-default = 恢复继承的品牌徽标偏好设置
avatars-domain-icon = { $name } 域名图标 · 此图标不验证发件人身份
avatars-favorites-first = 在收件人建议中优先显示收藏的联系人
avatars-favorites-first-help = 输入收件人时，你加星标的联系人会排在其他匹配项之前。只改变顺序，不会隐藏任何建议。
avatars-favorites-first-default = 恢复继承的收藏偏好设置

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
avatars-require-auth = 品牌标志需要认证

avatars-require-auth-always = 始终（需要通过检查）

avatars-require-auth-default = 恢复继承的身份验证首选项

avatars-require-auth-help = 选择何时可以用公认的公司徽标替换发件人旁边的姓名缩写。从不：只要发件人的域匹配，就显示它。仅当检查失败时（默认）：显示它，除非消息失败DKIM或DMARC。始终：仅当邮件通过 DKIM 且对齐的 DMARC 检查该公司自己的域时才显示，因此尚未检查的邮件会显示缩写。域匹配包括子域，例如mail.example.com。此首选项与您的帐户一起保存，并且可以由您的管理员保留。

avatars-require-auth-known-failure = 仅当检查失败时

avatars-require-auth-never = 从不（仅匹配域）

avatars-unverified-sender = 未经验证的发件人 · 没有显示品牌图标，因为此邮件未通过身份验证检查
