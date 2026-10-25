# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: settings. Source: ../en/settings.ftl

settings-title = 设置
settings-appearance = 外观
settings-close = 关闭设置

settings-theme = 主题
settings-density = 密度
settings-accent = 强调色
settings-font = 界面字体
settings-layout = 布局

# Density options
settings-density-compact = 紧凑
settings-density-cozy = 适中
settings-density-relaxed = 宽松

# Interface-font options
settings-font-default = 默认
settings-font-system = 系统
settings-font-serif = 衬线
settings-font-mono = 等宽

# Layout options
settings-layout-default = 默认
settings-layout-ribbon = 功能区

## Account settings surface (t16 e15) — 2FA, sessions, signatures, identities,
## notification rules, saved searches, device preferences. Shared action labels
## first, then one block per screen.

settings-cancel = 取消
settings-edit = 编辑
settings-delete = 删除
settings-save = 保存
settings-saved = 已保存

# Two-factor authentication (S1)
settings-2fa-title = 双重身份验证
settings-2fa-intro = 添加第二重验证，使登录时不仅需要密码。
settings-2fa-policy-required = 你的组织要求此账户使用第二重验证。
settings-2fa-enabled = 已启用
settings-2fa-recovery-title = 恢复代码
settings-2fa-recovery-once = 请立即保存这些代码。它们只显示一次，之后无法再次获取。每个代码只能使用一次。
settings-2fa-recovery-ack = 我已保存这些代码
settings-2fa-totp-title = 身份验证器应用
settings-2fa-totp-enrol = 设置身份验证器应用
settings-2fa-totp-scan = 将此密钥添加到身份验证器应用，然后输入它显示的代码。
settings-2fa-totp-uri-link = 在身份验证器应用中打开
settings-2fa-totp-code-label = 身份验证器代码
settings-2fa-totp-confirm = 确认
settings-2fa-totp-disable = 移除身份验证器
settings-2fa-passkey-title = 通行密钥
settings-2fa-passkey-remove = 移除
settings-2fa-passkey-unsupported = 此浏览器不支持通行密钥。
settings-2fa-passkey-label-placeholder = 通行密钥名称（可选）
settings-2fa-passkey-add = 添加通行密钥
settings-2fa-recovery-remaining = 剩余 { $count } 个恢复{ $count ->
       *[other] 代码
    }
settings-2fa-recovery-regenerate = 生成新的恢复代码
settings-2fa-recovery-code-label = 恢复代码
settings-2fa-error-generic = 操作未成功。请重试。
settings-2fa-error-code = 该代码未通过验证。
settings-2fa-error-passkey = 无法注册通行密钥。

# Two-factor challenge at sign-in (S1 web half)
settings-2fa-challenge-title = 双重身份验证
settings-2fa-challenge-intro = 确认第二重验证以完成登录。
settings-2fa-challenge-failed = 验证未通过。请重试。
settings-2fa-challenge-use-passkey = 使用通行密钥
settings-2fa-challenge-method = 选择验证方式
settings-2fa-challenge-totp-tab = 身份验证器代码
settings-2fa-challenge-recovery-tab = 恢复代码
settings-2fa-challenge-verify = 验证

# Active sessions (S11)
settings-sessions-title = 活动会话
settings-sessions-intro = 当前登录此账户的会话。
settings-sessions-last-seen = 上次活动时间：{ $when }
settings-sessions-revoke = 退出登录
settings-sessions-current = 当前会话
settings-sessions-revoke-others = 退出其他 { $count } 个{ $count ->
       *[other] 会话
    }
settings-sessions-error = 无法更改该会话。

# Signatures (W12)
settings-sig-title = 签名
settings-sig-intro = 附加在你发送的邮件末尾的文本。可以将一个签名设为默认。
settings-sig-default = 默认
settings-sig-new = 新建签名
settings-sig-name-label = 名称
settings-sig-body-label = 签名
settings-sig-default-label = 设为默认
settings-sig-error-name = 请为签名命名。
settings-sig-error-generic = 无法保存签名。

# Identities
settings-ident-title = 发件身份
settings-ident-intro = 发送邮件时使用的名称和地址。每个发件身份都可以使用自己的签名。
settings-ident-new = 新建发件身份
settings-ident-name-label = 显示名称
settings-ident-email-label = 电子邮件地址
settings-ident-replyto-label = 回复地址（可选）
settings-ident-signature-label = 默认签名
settings-ident-signature-none = 无
settings-ident-error-fields = 请输入名称和有效的电子邮件地址。
settings-ident-error-generic = 无法保存发件身份。

# Notification rules + quiet hours (W15)
settings-notif-title = 通知
settings-notif-intro = 选择哪些新邮件会通知你，以及何时保持安静。
settings-notif-enabled-label = 收到新邮件时显示通知
settings-notif-quiet-title = 免打扰时段
settings-notif-quiet-enabled-label = 在设定时间内静音通知
settings-notif-quiet-start = 从
settings-notif-quiet-end = 至
settings-notif-rules-title = 规则
settings-notif-rule-match-placeholder = 发件人、文件夹或主题包含…
settings-notif-rule-action-label = 操作
settings-notif-rule-notify = 通知
settings-notif-rule-mute = 静音
settings-notif-rule-add = 添加规则
settings-notif-error = 无法保存通知设置。

# Saved searches → search folders (W13)
settings-search-title = 已保存的搜索
settings-search-intro = 将已保存的搜索作为文件夹显示在文件夹列表中。
settings-search-empty = 你还没有已保存的搜索。
settings-search-as-folder = 将 { $name } 显示为文件夹
settings-search-as-folder-label = 显示为文件夹
settings-search-error = 无法更改已保存的搜索。

# Keyboard shortcuts (W14)
settings-kbd-title = 键盘快捷键
settings-kbd-intro = 选择一套快捷键。更改仅应用于此设备。
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = 关闭（无快捷键）
settings-kbd-action-compose = 撰写
settings-kbd-action-archive = 归档
settings-kbd-action-reply = 回复
settings-kbd-action-next = 下一封邮件
settings-kbd-action-previous = 上一封邮件
settings-kbd-action-search = 搜索

# Offline cache (W16)
settings-offline-title = 离线缓存
settings-offline-intro = 在此设备上保留多少邮件供离线使用，以及如何回收空间。
settings-offline-budget-label = 缓存上限（MB）
settings-offline-retention-label = 保留时长（天）
settings-offline-strategy-label = 达到上限时
settings-offline-lru = 移除最近最少使用的内容
settings-offline-oldest = 优先移除最旧的内容
settings-offline-manual = 仅在我手动清除时
settings-offline-purge = 立即清除离线缓存

# Interface direction (W20)
settings-dir-title = 界面方向
settings-dir-intro = 跟随语言设置，或强制指定方向。从右到左会镜像整个布局。
settings-dir-auto = 自动
settings-dir-ltr = 从左到右
settings-dir-rtl = 从右到左
settings-dir-preview-title = 预览
settings-dir-preview-body = 当方向为从右到左时，此文本会镜像显示。
