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
settings-mail-features = 邮件功能
settings-mail-feature-allowSend = 允许发送邮件
settings-mail-feature-allowSend-help = 关闭后，“发送”及其快捷键将被禁用。仍可保存草稿。
settings-mail-feature-allowReply = 允许回复和全部回复
settings-mail-feature-allowReply-help = 关闭后，不再提供“回复”和“全部回复”操作及其快捷键。
settings-mail-feature-allowForward = 允许转发
settings-mail-feature-allowForward-help = 关闭后，不再提供“转发”操作及其快捷键。
settings-mail-feature-allowDownload = 允许下载邮件和附件
settings-mail-feature-allowDownload-help = 关闭后，不再提供邮件和附件的下载。
settings-mail-feature-allowPrint = 允许打印
settings-mail-feature-allowPrint-help = 关闭后，不再提供“打印”操作。
settings-mail-feature-showSpam = 显示垃圾邮件文件夹
settings-mail-feature-showSpam-help = 关闭后，垃圾邮件文件夹和标记为垃圾邮件的邮件将在文件夹列表和搜索结果中隐藏。不会删除任何内容。
settings-mail-feature-showAuthStatus = 显示发件人验证结果
settings-mail-feature-showAuthStatus-help = 在阅读窗格中显示邮件的 SPF、DKIM 和 DMARC 结果。
settings-mail-feature-closeOnDelete = 删除邮件时将其关闭
settings-mail-feature-closeOnDelete-help = 当打开的邮件被删除或移至废纸篓时，关闭该邮件所在的标签页或窗口。

# -- Machine translation (2026-11): added to match en ---------------------
settings-breadcrumb = 设置路径
settings-crumb-loading = 正在加载名称
settings-crumb-new = 新建
settings-crumb-edit = 编辑
settings-badge-color = 未读徽章颜色
settings-badge-color-accent = 与强调色相同
settings-badge-color-accent-lighter = 较浅的强调色
settings-badge-color-accent-darker = 较深的强调色
settings-badge-color-option = { $color }徽章
settings-appearance-theme-card = 主题与切换
settings-appearance-theme-card-help = 选择主题,以及是跟随系统还是跟随一天中的时间。
settings-appearance-colours-card = 颜色
settings-appearance-colours-card-help = 用于操作和选中项的强调色,以及未读计数器的颜色。
settings-appearance-text-card = 文字与布局
settings-appearance-text-card-help = 界面的紧凑程度、使用的字体以及邮件的排版方式。
settings-sig-create-variant = 创建变体
settings-sig-error-existing = 已存在同名的签名。请选择其他名称。
settings-sig-save-failed = 无法保存签名。{ $reason }
settings-ident-signature-new = 新邮件
settings-ident-signature-account = 使用账户签名
settings-ident-signature-replies = 回复
settings-ident-signature-inherit = 使用新邮件的签名
settings-ident-signature-unavailable = 签名不可用:{ $name }
settings-ident-signature-help = 在“签名”中创建签名变体,然后在此处指定。同一个变体可同时用于两者。
settings-ident-save-failed = 无法保存身份。{ $reason }
settings-search-save-failed = 无法保存已存搜索。{ $reason }
settings-kbd-preset-title = 快捷键预设
settings-editor-discard-title = 放弃未保存的更改?
settings-editor-discard-detail = 你的更改尚未保存。可继续编辑,或放弃此草稿并离开编辑器。
settings-editor-keep = 继续编辑
settings-editor-discard = 放弃更改
settings-editor-missing = 此项目已不存在。请返回列表并选择其他项目。
settings-editor-back = 返回列表
settings-sig-edit-title = 编辑签名
settings-ident-edit-title = 编辑身份
settings-sections = 设置分区
settings-category-account = 账户
settings-category-mail = 邮件
settings-category-interface = 界面
settings-category-integrations = 集成
settings-category-about = 关于
settings-nav-preferences = 键盘与显示
settings-nav-rules = 邮件规则
settings-nav-access = API 访问
settings-nav-metadata = 服务器信息
settings-folders-title = 文件夹与侧边栏
settings-folders-help = 将文件夹固定在其他文件夹之上、选择图标,并通过拖动或移动按钮在各组内排列文件夹。这些选择会与此账户同步,不会移动邮件或重命名文件夹。
settings-folders-pin = 固定 { $name }
settings-folders-icon = { $name } 的图标
settings-folders-automatic = 自动文件夹角色
settings-folders-drag = 拖动 { $name } 以重新排序
settings-folders-up = 上移 { $name }
settings-folders-down = 下移 { $name }
settings-folders-sync-idle = 登录后才能同步文件夹偏好设置。
settings-folders-sync-loading = 正在加载账户偏好设置…
settings-folders-sync-saving = 正在保存账户偏好设置…
settings-folders-sync-synced = 偏好设置已与此账户同步。
settings-folders-sync-error = 无法同步偏好设置。你的更改会保留在本次会话中,并在连接恢复后重试。
settings-folder-icon-folder = 文件夹
settings-folder-icon-star = 星标
settings-folder-icon-flag = 旗标
settings-folder-icon-tag = 标签
settings-folder-icon-clock = 时钟
settings-folder-icon-calendar = 日历
settings-folder-icon-file = 文档
settings-folder-icon-image = 图片
settings-folder-icon-lock = 锁
settings-folder-icon-users = 人员
settings-folder-icon-archive = 归档
settings-folder-icon-mail = 信封
settings-folder-icon-attachment = 附件
settings-folder-icon-check = 对勾
settings-folder-icon-alert = 警示
settings-search-new = 新建搜索文件夹
settings-search-editor = 搜索文件夹
settings-search-editor-help = 邮件必须满足所有条件。留空的条件将被忽略。搜索文件夹会显示匹配的邮件,但不会移动或复制它们。
settings-search-name = 文件夹名称
settings-search-name-required = 请输入此搜索文件夹的名称。
settings-search-invalid = 此已存搜索的筛选条件无效,无法安全编辑。
settings-search-saved = 已保存搜索文件夹“{ $name }”。
settings-search-field-text = 搜索所有邮件文本
settings-search-field-from = 发件人包含
settings-search-field-to = 收件人包含
settings-search-field-subject = 主题包含
settings-search-field-body = 邮件正文包含
settings-search-field-filename = 附件文件名包含
settings-search-field-inMailbox = 限定邮箱标识符(可选)
settings-search-field-after = 接收时间晚于(UTC 日期和时间)
settings-search-field-before = 接收时间早于(UTC 日期和时间)
settings-search-attachment = 附件
settings-search-any = 任何邮件
settings-search-with-attachments = 含附件
settings-search-without-attachments = 不含附件
settings-search-unread = 仅未读邮件
settings-search-flagged = 仅已标记邮件
settings-sessions-search = 搜索账户、设备或 IP
settings-sessions-account = 账户
settings-sessions-device = 设备与客户端
settings-sessions-location = IP 与最近已知国家/地区
settings-sessions-location-unavailable = 国家/地区不可用
settings-sessions-activity = 活动
settings-sessions-actions = 操作
settings-sessions-revoke-device = 退出 { $device }
settings-font-inter = Inter
settings-font-manrope = Manrope
settings-font-roboto = Roboto
settings-font-open-sans = Open Sans
settings-font-jost = Jost
settings-compose-title = 撰写与回复
settings-quote-style = 回复与转发的头部样式
settings-quote-classic = 经典引用回复
settings-quote-outlook = Outlook 样式头部
settings-quote-help = Outlook 样式会在富文本和纯文本中,于原始邮件上方添加发件人、发送时间、收件人、抄送和主题。现有草稿不受影响。
settings-external-warning = 外部收件人警告
settings-external-off = 关闭
settings-external-warn = 显示警告
settings-external-confirm = 发送前要求确认
settings-external-help = 将收件人、抄送和密送与所选身份的域名及受信任域名列表进行比对。这只保护从此撰写窗口发出的邮件,不是服务器的投递规则。
settings-trusted-domains = 受信任的收件人域名
settings-trusted-domains-help = 最多输入 100 个确切的域名,每行一个。子域名必须单独列出。发送身份自己的域名始终受信任。不接受 URL、电子邮件地址和通配符。
settings-trusted-domains-invalid = 最多输入 100 个有效的域名,不含 URL、电子邮件地址或通配符。
settings-trusted-domains-save = 保存受信任的域名
settings-shortcuts-enabled = 启用应用快捷键
settings-shortcuts-action = 操作
settings-shortcuts-binding = 按键绑定
settings-shortcuts-edit = 编辑快捷键
settings-shortcuts-edit-action = 编辑“{ $action }”快捷键
settings-shortcuts-help = 填写 inherit 表示沿用所选预设,disabled 表示不设快捷键,或填写组合键,如 Ctrl+Shift+K。在 macOS 上 Ctrl 表示 Cmd。冲突的绑定不会生效。输入框和对话框保留其常规按键;浏览器保留的组合键可能不可用。
settings-shortcuts-invalid = 请输入 inherit、disabled,或带有可选 Ctrl、Alt、Shift 修饰键的有效单个按键。
settings-shortcuts-conflict = 有两个操作使用了相同的快捷键。请更改其中一个绑定;冲突的快捷键不会运行。
settings-shortcuts-discard = 放弃未保存的快捷键更改?
settings-shortcuts-save = 保存快捷键
settings-shortcuts-back = 返回快捷键
settings-shortcuts-off = 你的“关闭”预设会禁用所有快捷键,包括自定义绑定。请在“偏好设置”中选择其他预设后再使用这些绑定。
settings-kbd-action-replyAll = 全部回复
settings-kbd-action-forward = 转发
settings-kbd-action-refresh = 刷新邮件
settings-kbd-action-settings = 打开设置
settings-kbd-action-trash = 移到废纸篓
settings-kbd-action-markRead = 标记为已读
settings-kbd-action-markUnread = 标记为未读
settings-kbd-action-spam = 标记为垃圾邮件
settings-kbd-action-zoomIn = 放大邮件
settings-kbd-action-zoomOut = 缩小邮件
settings-kbd-action-zoomReset = 重置邮件缩放
settings-kbd-action-renameFolder = 重命名文件夹
settings-kbd-action-openInTab = 在单独标签页中打开邮件
settings-notif-device-title = 此设备上的权限
settings-notif-device-help = 你的通知偏好设置跟随你的账户。浏览器权限属于此设备;Mailwoman 只会在你选择允许时才请求。
settings-notif-ask-permission = 在新设备上提示申请通知权限
settings-notif-ask-permission-help = 当此设备既未允许也未阻止通知时,显示权限提示。默认启用;此偏好设置会立即保存。
settings-notif-permission-default = 尚未请求浏览器权限。
settings-notif-permission-granted = 此设备上已允许浏览器通知。
settings-notif-permission-denied = 通知已被阻止。请在浏览器设置中更改此网站的权限以允许通知。
settings-notif-permission-unsupported = 此浏览器在此处不支持系统通知。
settings-notif-permission-request = 在此设备上允许通知
settings-notif-permission-error = 浏览器无法请求通知权限。
settings-notif-timezone = 免打扰时段的时区(IANA)
settings-notif-timezone-device = 使用此设备的时区
settings-permissions-background-title = 此浏览器上的后台通知
settings-permissions-background-help = 请先保存通知偏好设置并允许浏览器通知。注册此浏览器后,即使应用已关闭,也能接收账户活动。
settings-permissions-push-unsupported = 此浏览器不支持后台 Web Push。
settings-permissions-push-registered = 此浏览器已有推送订阅。请注册以将其绑定到当前账户。
settings-permissions-push-unregistered = 此浏览器没有推送订阅。
settings-permissions-push-register = 注册后台通知
settings-permissions-push-remove = 移除浏览器订阅
settings-permissions-push-policy = 管理员不允许后台推送通知。
settings-permissions-storage-title = 浏览器存储
settings-permissions-storage-help = 持久存储可在浏览器授予时,防止此网站的本地数据被自动清除。它仅限于本设备,不会改变服务器的保留策略。你可以在浏览器设置中清除网站数据;尚未发送的离线更改可能会丢失。
settings-permissions-storage-unsupported = 此浏览器不支持持久存储请求。
settings-permissions-storage-persistent = 浏览器已为此网站授予持久存储。
settings-permissions-storage-evictable = 浏览器可能会自动清除此网站的本地数据。
settings-permissions-storage-usage = 当前网站存储:{ $mib } MiB。
settings-permissions-storage-request = 请求持久存储
settings-permissions-storage-policy = 管理员不允许持久存储请求。
settings-permissions-error = 无法完成浏览器操作。请检查已保存的账户设置以及此网站的浏览器权限。
settings-notif-background-rules = 内容规则只在 Mailwoman 打开时评估。配置了任何内容规则时,后台系统通知会被抑制,因为不透明的后台唤醒不包含邮件详情。
settings-nav-regional = 区域设置
settings-time-title = 时区
settings-time-help = 选择邮件时间的显示方式,以及用于日程安排的可选对照时区。绝不会根据电子邮件地址推断位置。
settings-time-displayZone = 显示时区
settings-time-displayZone-help = 使用 local 表示此设备的时区,或使用 IANA 名称,例如 Europe/Lisbon。
settings-time-comparisonZone = 对照时区
settings-time-comparisonZone-help = 输入 IANA 名称,例如 America/New_York;留空则隐藏对照。
settings-time-helpers = 显示时区辅助信息
settings-time-save = 保存时区
settings-time-invalid = 请输入有效的 IANA 时区。local 仅可用于显示时区,对照时区可留空。
settings-time-discard = 放弃未保存的时区更改?
settings-folders-name = 文件夹
settings-folders-actions = 操作
settings-folders-edit = 编辑 { $name } 的外观
settings-folders-icon-help = 从五十个主题图标中选择一个,或保持根据文件夹用途自动选择。此设置跟随你的账户,不会重命名邮箱。
settings-folders-save = 保存文件夹外观
settings-folders-back = 返回文件夹
settings-folders-discard = 放弃未保存的文件夹外观更改?
settings-folder-labels = 文件夹标签
settings-folder-labels-help = 用最多 32 个账户标签整理此文件夹。这不会为其中的邮件添加标签或移动邮件。请在“标签”中管理标签名称和颜色。
settings-folders-leave-pending = 这些更改仍在排队等待同步。要离开此编辑器吗?
settings-folders-retry = 重试保存
settings-folder-role-title = 特殊文件夹
settings-folder-role-help = 为每个邮件角色选择实际使用的文件夹。“自动”使用服务器发现。“使用管理员默认值”会移除你的覆盖设置。目标冲突或缺失时,该角色会被禁用,而不会改选其他文件夹。

# -- Machine translation (2026-11): added to match en ---------------------
cache-clear = 立即清除缓存的邮件
settings-compose-window = 新邮件打开方式
settings-compose-window-tab = 工作区标签页
settings-compose-window-new = 新窗口
settings-compose-window-help = 如果浏览器阻止新窗口,撰写窗口会在工作区标签页中打开。
settings-compose-read-receipt = 始终请求已读回执
settings-compose-delivery-receipt = 始终请求送达回执
settings-compose-reply-copy = 将回复的副本保存在原文件夹
settings-compose-receipts-help = 回执只是请求。收件人或其邮件服务器可能拒绝或忽略。
settings-compose-forward = 默认转发格式
settings-compose-forward-inline = 内嵌邮件文本
settings-compose-forward-attachment = 原邮件作为 .eml 附件
settings-compose-saving = 正在保存撰写偏好设置…
settings-compose-saved = 撰写偏好设置已保存
settings-ident-organization = 组织
settings-ident-auto-bcc = 自动密送
settings-ident-auto-bcc-help = 使用此身份发送时,最多可添加 20 个密送地址。地址之间用逗号分隔。
settings-ident-pgp-key = OpenPGP 密钥
settings-ident-pgp-none = 使用默认密钥
settings-ident-pgp-unavailable = 已保存的密钥不可用 — 请选择其他密钥
settings-ident-pgp-help = 从你现有的 OpenPGP 签名密钥中选择一个。私钥材料保留在你的密钥库中。
settings-ident-error-optional = 请检查回复地址、自动密送地址、组织和所选密钥。
settings-compose-open-tracking = 默认记录邮件打开情况
settings-compose-open-tracking-help = 向符合条件的格式化邮件添加跟踪图片。打开记录取决于收件人是否加载图片,并不能证明邮件已被阅读。管理员可以禁用跟踪或锁定此默认值。
settings-compose-open-tracking-inherit = 使用继承的跟踪默认值
settings-reply-images = 回复和转发中的图片
settings-reply-images-ask = 每次询问
settings-reply-images-allow = 始终包含
settings-reply-images-trusted = 受信任的发件人域名包含,其他询问
settings-reply-images-deny = 从不包含
settings-reply-images-help = 选择回复或转发时是否携带原邮件的图片。包含的图片会作为附件复制到草稿中。
settings-reply-image-domains = 受信任的图片发件人域名
settings-reply-image-domains-help = 最多输入 200 个确切的发件人域名,每行一个。当上面选择“受信任的域名”时使用。
settings-reply-image-domains-save = 保存受信任的图片域名
settings-ident-group-basics = 基本信息
settings-ident-group-replies = 回复处理
settings-ident-group-signature = 签名
settings-ident-group-sending = 发送选项
settings-ident-name-required = 请输入显示名称。
settings-ident-email-invalid = 请输入有效的电子邮件地址,例如 name@example.org。
settings-ident-email-duplicate = 你已有使用此地址的身份。
settings-ident-replyto-invalid = 请输入有效的电子邮件地址,或留空。
settings-ident-replyto-help = 对使用此身份发送的邮件的回复,会发送到此地址,而不是发件人地址。
settings-ident-bcc-invalid = 请检查密送列表:使用有效的地址,以逗号分隔,最多 20 个。
settings-ident-delete-title = 删除身份 { $email }?
settings-ident-delete-body = 你将无法再使用此身份发送邮件。草稿和已发送的邮件不受影响。
settings-ident-empty = 你还没有任何身份。添加一个,即可在撰写时选择其名称、地址和签名。
settings-sig-delete-title = 删除签名 { $name }?
settings-sig-delete-body = 使用它的身份将改用账户签名。已发送的邮件不受影响。
settings-sig-deleted = 签名已删除。
settings-reply-from = 回复所用身份
settings-reply-from-match = 邮件所发往的地址
settings-reply-from-default = 始终使用我的默认身份
settings-reply-from-ask = 多个身份匹配时询问我
settings-reply-from-keep = 保持已选择的身份
settings-reply-from-help = 回复或转发时,选择使用哪个身份。如果没有身份与邮件所发往的地址匹配,则使用你的默认身份。像 name+tag@example.org 这样的加号地址会匹配 name@example.org。
settings-ident-default-badge = 默认
settings-ident-make-default = 设为默认
settings-ident-default-set = { $email } 现在是你的默认身份。
settings-ident-create-variant = 创建变体
settings-ident-variant-count = { $count } { $count ->
       *[other] 个变体
    }
settings-ident-inherits = 以下字段沿用 { $base }:{ $fields }
settings-ident-field-replyTo = 回复地址
settings-ident-field-autoBcc = 自动密送
settings-ident-field-organization = 组织
settings-ident-field-pgpKeyId = OpenPGP 密钥
settings-ident-field-signatureName = 签名
settings-ident-delete-default-body = 这是你的默认身份。剩余身份中最早创建的将成为默认身份。你将无法再使用此身份发送邮件。草稿和已发送的邮件不受影响。
settings-ident-variants-choice-title = 删除 { $email }?
settings-ident-variants-choice-body = 此身份有 { $count } { $count ->
       *[other] 个变体
    }。可以一并删除,也可以将{ $count ->
       *[other] 它们
    }保留为独立的身份。保留的变体会复制它从此身份继承的设置。
settings-ident-delete-with-variants = 删除身份及变体
settings-ident-delete-keep-variants = 删除身份,保留变体
settings-ident-new-variant = 新建变体
settings-ident-edit-variant-title = 编辑变体
settings-ident-variant-intro = { $base } 的变体。字段留空则使用该身份的值。
settings-ident-label-label = 变体标签
settings-ident-label-required = 请为此变体输入标签,例如“新闻通讯”。
settings-ident-inherit-hint = 留空:使用基础身份的 { $value }。
settings-ident-inherit-none = 无
settings-check-policy-allow = 直接发送,不询问
settings-check-policy-warn = 每次询问
settings-check-policy-warn-remember = 询问,并允许我不再询问
settings-check-policy-deny = 阻止发送
settings-check-invalid = 无效的收件人地址
settings-check-attachment = 提到附件但未添加
settings-check-subject = 主题为空
settings-check-help = 检查发现问题时该怎么做。“询问”始终会提供“仍然发送”;只有“阻止发送”会拦下邮件。
settings-external-warn-remember = 询问,并允许我不再询问
settings-external-deny = 阻止发送
settings-auto-chip = 输入空格、逗号或分号时完成收件人输入
settings-auto-chip-help = 有效的地址会变成完整的收件人,并自动为你添加分隔符。
settings-recipient-separator = 收件人分隔符
settings-recipient-separator-comma = 逗号
settings-recipient-separator-semicolon = 分号
settings-header-collapse = 邮件头(收件人、抄送、主题)
settings-header-collapse-expanded = 始终显示
settings-header-collapse-collapsed = 折叠为摘要行
settings-header-collapse-auto = 撰写正文时折叠
settings-header-collapse-help = 折叠后,邮件头只显示一行,例如“收件人:Ana +2”,点击它或有字段需要注意时会展开。
settings-retention-title = 保留
settings-folder-icon-camera = 相机
settings-folder-icon-music = 音乐
settings-folder-icon-film = 电影
settings-folder-icon-lightbulb = 创意
settings-folder-icon-rocket = 火箭
settings-folder-icon-trophy = 奖杯
settings-folder-icon-leaf = 叶子
settings-folder-icon-globe = 地球
settings-folder-icon-cloud = 云
settings-folder-icon-bookmark = 书签
settings-folder-icon-wallet = 钱包
settings-folder-icon-medical = 医疗
settings-folder-icon-bell = 铃铛
settings-folder-icon-key = 钥匙
settings-folder-icon-receipt = 收据
settings-folder-icon-cart = 购物车
settings-folder-icon-package = 包裹
settings-folder-icon-wrench = 工具
settings-folder-icon-chart = 图表
settings-folder-icon-team = 团队
settings-folder-icon-store = 商店
settings-folder-icon-office = 办公室
settings-folder-icon-laptop = 笔记本电脑
settings-folder-icon-crown = 皇冠
settings-folder-icon-family = 家庭
settings-folder-icon-group-general = 通用
settings-folder-icon-group-mail = 邮件
settings-folder-icon-group-work = 工作
settings-folder-icon-group-finance = 财务与购物
settings-folder-icon-group-travel = 旅行与家居
settings-folder-icon-group-personal = 个人与娱乐
settings-folder-icon-group-tech = 科技与工具
settings-folder-icon-search-label = 搜索图标
settings-folder-icon-no-match = 没有匹配的图标

# -- Machine translation (2026-11): added to match en ---------------------
settings-folder-role-auto = 自动发现
settings-folder-role-inherit = 使用管理员默认值
settings-folder-role-path = 已配置的路径:{ $path }
settings-folder-role-missing = 已配置的文件夹不可用
settings-folder-role-conflict = 此目标缺失,或与另一个角色冲突。在映射修正之前,该角色不可用。
settings-folder-role-refresh-error = 已保存,但无法刷新文件夹列表。使用新映射前请先刷新文件夹。
settings-folder-role-unsupported = 这些覆盖设置需要原生邮件文件夹。上游 JMAP 账户沿用其服务器的文件夹角色。
settings-folder-role-inbox = 收件箱
settings-folder-role-drafts = 草稿
settings-folder-role-sent = 已发送邮件
settings-folder-role-trash = 废纸篓
settings-folder-role-junk = 垃圾邮件
settings-folder-role-archive = 归档
settings-sessions-select = 选择
settings-sessions-select-device = 选择 { $device } 的会话
settings-sessions-select-visible = 选择所有可见的其他会话
settings-sessions-current-protected = 此会话保持登录状态,无法在此处选择。请使用“退出登录”来结束它。
settings-sessions-selected-title = 退出所选会话({ $count })
settings-sessions-selected-confirm = 仅结束所选的其他会话。此会话保持登录。失败或跳过的请求会被报告,不会自动重试。
settings-sessions-selected-action = 退出所选会话
settings-sessions-cancel = 取消
settings-sessions-selected-result = 已退出:{ $revoked }。此前已结束:{ $unchanged }。无法确认:{ $failed }。未尝试:{ $skipped }。
settings-sessions-refresh-failed = 无法刷新会话列表。选择更多会话之前请重新加载。
settings-sig-format = 签名格式
settings-sig-format-plain = 纯文本
settings-sig-format-html = 富文本(HTML)
settings-sig-html-help = HTML 限制为 2 MiB,保存和发送前会进行净化。远程图片和活动内容会被移除。
settings-sig-html-source = 编辑 HTML 源码
settings-sig-editor-loading = 正在加载签名编辑器…
settings-sig-rich-body = 富文本签名
settings-sig-plain-alternative = 纯文本备选
settings-sig-plain-help = 纯文本邮件使用此文本。在你自定义之前,富文本的编辑会更新它;HTML 源码的编辑会保留你的备选内容。
settings-sig-account-changed = 你登录的账户已更改。请在正确的账户中重新打开此签名。
settings-sig-image-type = { $name } 不是 PNG、JPEG、GIF 或 WebP 图片。
settings-sig-too-large = 此签名的 HTML 超过 2 MiB。请移除一张图片或部分格式。
settings-sig-images-help = 图片存储在签名中,并作为每封邮件的内嵌图片发送。
settings-folder-icon-inbox = 收件箱
settings-folder-icon-sent = 已发送
settings-folder-icon-edit = 编辑
settings-folder-icon-delete = 删除
settings-folder-icon-spam = 垃圾邮件
settings-folder-icon-search = 搜索
settings-folder-icon-pin = 图钉
settings-folder-icon-link = 链接
settings-folder-icon-shield = 盾牌
settings-folder-icon-unlock = 解锁
settings-folder-icon-fileAudio = 音频
settings-folder-icon-fileVideo = 视频
settings-folder-icon-filePdf = PDF
settings-folder-icon-fileCode = 代码
settings-folder-icon-print = 打印
settings-folder-icon-download = 下载
settings-folder-icon-preferences = 偏好设置
settings-folder-icon-dashboard = 仪表板
settings-folder-icon-analytics = 分析
settings-folder-icon-nextcloud = 云文件
settings-folder-icon-domains = 域名
settings-folder-icon-twofactor = 身份验证
settings-folder-icon-assistant = 助手
settings-folder-icon-integrations = 集成
settings-folder-icon-logs = 日志
settings-folder-icon-home = 主页
settings-folder-icon-briefcase = 公文包
settings-folder-icon-heart = 心形
settings-folder-icon-shopping = 购物
settings-folder-icon-education = 教育
settings-folder-icon-travel = 旅行
settings-folder-icon-coffee = 咖啡
settings-folder-icon-gift = 礼物
settings-folder-icon-bank = 银行
settings-folder-icon-book = 书籍
settings-nav-keyboard-display = 键盘与显示
settings-2fa-passkey-policy-disabled = 管理员已为此账户禁用通行密钥登录和注册。现有凭据仍会列出,以便你管理或移除。
settings-mail-interactions = 邮件选择与移动
settings-mail-modifier-selection = 使用修饰键选择邮件
settings-mail-modifier-selection-help = Ctrl 或 Command 可切换单独的邮件。Shift 选择一个范围,Shift 加方向键可扩展范围。关闭此项时,仍可使用可见的选择按钮。
settings-mail-drag-folder = 将邮件拖到文件夹
settings-mail-drag-folder-help = 拖动已选中的邮件会移动整个选择,拖动未选中的邮件只会移动该邮件。只接受此账户中的文件夹。触屏和键盘用户可改用选择工具栏。
settings-search-sections = 搜索设置
settings-search-no-sections = 没有匹配的设置分区。
settings-time-local = 本设备的时区
settings-time-none = 无对照时区
settings-folder-counts = 文件夹计数
settings-folder-counts-none = 隐藏计数
settings-folder-counts-unread = 未读邮件
settings-folder-counts-total = 邮件总数
settings-folder-counts-both = 未读邮件和邮件总数
settings-folder-counts-unreadIfAny = 未读邮件,仅在有未读时显示
settings-folder-count-scope = 显示计数的文件夹
settings-folder-count-scope-all = 所有文件夹
settings-folder-count-scope-main = 仅主要文件夹
settings-folder-count-scope-help = 主要文件夹包括收件箱、草稿、已发送、归档、垃圾邮件、废纸篓、发件箱和其他特殊用途文件夹。自定义文件夹及其子文件夹不显示计数。
settings-account-unread-badge = 在邮箱名称旁显示未读数
settings-folder-counts-compact = 收件箱以外使用较小的徽章
settings-focused-inbox = 重点收件箱
settings-focused-inbox-help = 在列表上方将收件箱分为“重点”和“其他”标签页。新闻通讯和自动发件人的邮件会进入“其他”。其他文件夹和搜索结果显示所有邮件。
settings-focused-senders-title = 管理重点发件人
settings-focused-senders-help = 你选择始终移到“重点”或“其他”的发件人。移除后,将重新由自动分类决定。
settings-focused-senders-empty = 暂无发件人。请对邮件使用“始终移到重点”或“始终移到其他”。
settings-focused-senders-to-focused = 始终重点
settings-focused-senders-to-other = 始终其他
settings-focused-senders-remove = 移除
settings-focused-senders-remove-label = 移除 { $sender }
settings-focused-senders-remove-failed = 无法移除该发件人。请重试。
settings-folder-counts-reset = 沿用文件夹计数设置
settings-folder-counts-reset-size = 沿用徽章大小设置
settings-mail-resume-tabs = 恢复我打开的标签页
settings-mail-resume-tabs-help = 重新打开 Mailwoman 后,恢复账户所属的标签页引用。邮件正文和附件 URL 不会保存在工作区记录中。
settings-row-actions-mode = 邮件列表快捷操作
settings-row-actions-mode-open = 始终显示
settings-row-actions-mode-hover = 悬停时显示
settings-row-actions-mode-click = 点击时显示
settings-row-actions-mode-off = 关闭
settings-row-actions-mode-help = 每行邮件上快捷操作的显示方式。“悬停时显示”会显示三个控件和一个“更多”按钮;“点击时显示”会显示一个用于展开它们的按钮;在触屏上,悬停的行为与点击相同。“关闭”会从行中移除它们(上下文菜单和工具栏仍包含所有操作)。
settings-mail-feature-showGoToTop = 在邮件列表中显示“回到顶部”按钮
settings-mail-feature-showGoToTop-help = 添加一个按钮,可将邮件列表滚动回第一封邮件。默认关闭。
settings-mail-feature-showCreateFolderButton = 在“撰写”旁显示“新建文件夹”按钮
settings-mail-feature-showCreateFolderButton-help = 在侧边栏添加一个仅含图标的按钮,用于新建文件夹。默认关闭。
settings-draft-save = 账户草稿
settings-draft-save-help = 保存到账户的“草稿”文件夹,以便在其他设备上访问。保存时不会打扰你,只有失败时才会显示通知。不受支持的账户和受保护的已解密内容绝不会被悄悄上传。
settings-draft-save-mode = 自动保存
settings-draft-realtime = 输入停顿后
settings-draft-periodic = 定时保存
settings-draft-manual = 仅在我选择“保存”时
settings-draft-debounce = 输入停顿时间(毫秒)
settings-draft-interval = 保存间隔(秒)
settings-permissions-push-other-registration = 此浏览器有一个未注册到此账户的订阅。请先移除它,再重新注册。
settings-show-from = 撰写时显示发件人
settings-show-from-help = 一开始就显示“发件人”字段。你也可以在功能区中为单封邮件显示它。只接受已授权的发送地址。
settings-sessions-table = 活动会话表
settings-folders-table = 文件夹列表
settings-kbd-reference = 查看预设的按键绑定
settings-kbd-reference-help = 仅为预设参考。自定义绑定和管理员策略可能会改变实际生效的快捷键。
settings-sessions-client-details = 客户端详情
settings-2fa-passkey-rules = 你的组织只接受符合以下条件的通行密钥:
settings-2fa-passkey-rule-platform = 存储在此设备上(Windows Hello、Touch ID、Android 屏幕锁)
settings-2fa-passkey-rule-cross-platform = 位于安全密钥或手机上
settings-2fa-passkey-rule-user-verification = 会校验你的 PIN、指纹或面部
settings-2fa-passkey-rule-resident-key = 存储在验证器上(可发现)
settings-2fa-passkey-gap-allowed = 添加于当前策略生效之前,且不符合该策略。它仍可使用;请添加新的通行密钥并移除此项。
settings-2fa-passkey-gap-refused = 不符合当前策略,已不再接受用于登录。请添加新的通行密钥并移除此项。
settings-2fa-passkey-browser-refused-policy = 你的浏览器未找到符合组织通行密钥策略的验证器,或请求已被取消。
settings-2fa-policy-user-verification = 你的组织要求使用会校验你的 PIN、指纹或面部的通行密钥。此验证器没有验证你的身份。
settings-2fa-policy-platform = 你的组织只接受存储在此设备上的通行密钥(如 Windows Hello、Touch ID 或 Android 屏幕锁)。不接受安全密钥和手机。
settings-2fa-policy-cross-platform = 你的组织只接受安全密钥或手机通行密钥。不接受存储在此设备上的通行密钥。
settings-2fa-policy-attachment-unknown = 你的浏览器没有说明这是哪种验证器,因此无法根据组织的通行密钥策略进行检查。请尝试其他浏览器。
settings-2fa-policy-resident-key = 你的组织要求使用存储在验证器上的通行密钥。此验证器没有存储。
settings-2fa-policy-legacy = 此通行密钥添加于组织当前的通行密钥策略之前,已不再符合要求。请使用其他因素或恢复代码登录,然后添加新的通行密钥。
settings-2fa-policy-last-factor-blocked = 你的组织不允许移除你的最后一个第二重因素。请先添加其他登录方式。
settings-2fa-policy-last-factor-required = 你的组织要求使用双重身份验证。移除此项之前,请先添加其他登录方式。
settings-2fa-last-factor-hint = 请先添加其他登录方式。
settings-2fa-last-factor-tooltip = 请先添加其他登录方式。你的组织要求你的账户上至少有一个通行密钥或验证器应用。
settings-2fa-challenge-passkey-not-used = 未使用通行密钥:请求已取消或超时,或此验证器无法满足你的组织的要求(如 PIN、指纹或面部校验)。请重试,或使用其他方式。
settings-folders-hidden-help = 隐藏的文件夹及其子文件夹不会出现在侧边栏中。它们不会被删除,也不受访问限制。在此显示某个子文件夹时,也会显示其被隐藏的上级文件夹。收件箱始终可用。
settings-folders-collapse-user = 在侧边栏中折叠用户文件夹
settings-folders-show = 在侧边栏中显示 { $name }
settings-folders-hide = 从侧边栏中隐藏 { $name }
settings-folders-show-short = 显示
settings-folders-hide-short = 隐藏
settings-folders-search = 搜索文件夹
settings-folders-select = 选择文件夹:{ $name }
settings-folders-selected = 已选择的文件夹:{ $count }
settings-folders-select-visible = 选择可见项
settings-folders-select-all = 选择所有文件夹
settings-folders-clear = 清除选择
settings-folders-pin-selected = 固定所选项
settings-folders-unpin-selected = 取消固定所选项
settings-folders-show-selected = 显示所选项
settings-folders-hide-selected = 隐藏所选项
settings-folders-visible = 已显示
settings-folders-hidden = 已隐藏
settings-folders-pinned = 已固定
settings-folders-location = 位置
settings-folders-counts = { $unread } 封未读 / 共 { $total } 封
settings-folders-more = { $name } 的更多操作
settings-folders-empty = 没有与搜索匹配的文件夹。
settings-folders-root = 顶层
settings-folders-order-help = 拖动文件夹可在侧边栏中重新排序。已固定的文件夹会保持在一起。
cache-title = 邮件缓存与预加载
cache-help = 将最近访问的文件夹和邮件保留在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会把邮件标记为已读,也不会加载远程图片。
cache-enabled = 启用会话缓存
cache-mailboxes = 缓存文件夹的邮件列表
cache-messages = 缓存已打开的邮件
cache-freshSeconds = 新鲜度间隔(秒)
cache-maxAgeSeconds = 缓存最长时效(秒)
cache-maxEntries = 缓存条目上限
cache-maxBytes = 内存预算(字节)
cache-prefetchEnabled = 启用预测性预加载
cache-hoverEnabled = 指向文件夹时预加载
cache-focusEnabled = 文件夹获得键盘焦点时预加载
cache-dwellMs = 指针和焦点延迟(毫秒)
cache-idleEnabled = 空闲时预加载文件夹
cache-idleDelayMs = 空闲延迟(毫秒)
cache-maxConcurrency = 同时预加载的上限
cache-maxQueued = 排队预加载的上限
cache-preloadMessages = 允许预测性加载邮件正文
cache-excludedFolders = 不参与预加载的文件夹
cache-exclusions-help = 每行一条规则:role:trash、role:junk、role:inbox、role:drafts、role:sent、role:archive,或 id: 后跟确切的文件夹 ID。
cache-refreshOnOpen = 打开时刷新缓存内容

# -- Machine translation (2026-11): added to match en ---------------------
settings-sig-image-size = { $name } 放不下:含图片在内,签名的 HTML 限制为 2 MiB。

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
settings-badge-auth = 身份验证（DMARC）

settings-badge-auth-help = DMARC 结果；它会打开完整的安全详细信息。

settings-badge-density = 徽章尺寸

settings-badge-density-comfortable = 舒适

settings-badge-density-compact = 紧凑型

settings-badge-density-help = 每个徽章的高度和间距紧凑或舒适。

settings-badge-folder = 文件夹

settings-badge-folder-help = 消息存储的位置。仅当显着时才会显示垃圾和垃圾。

settings-badge-labels = 徽章标签

settings-badge-labels-help = 带有文本的图标，或单独的图标（完整文本保留在工具提示中并供屏幕阅读器使用）。

settings-badge-labels-icon = 仅图标

settings-badge-labels-text = 图标和文字

settings-badge-list = 邮件列表

settings-badge-list-help = 消息来自的邮件列表；它会打开订阅详细信息和操作。仅当您取消订阅后邮件不断到达时才会显着显示。

settings-badge-mode-always = 总是

settings-badge-mode-never = 从来没有

settings-badge-mode-notable = 仅当引人注目时

settings-badge-sanitized = 已消毒HTML

settings-badge-sanitized-help = HTML 消毒剂对消息做了什么。仅当某些内容被阻止时才会显着显示。

settings-badge-spam = 垃圾邮件分数

settings-badge-spam-help = 接收服务器写入消息标头的分数。

settings-folders-hide-empty = 隐藏侧边栏中的空文件夹

settings-folders-hide-empty-help = 当文件夹包含消息时，它们会重新出现。收件箱、搜索文件夹、计数未知的文件夹以及可见文件夹的父级保持显示。覆盖“属性”、“显示”下的各个文件夹；每个文件夹都保留在此列表中。

settings-folders-list-card = 文件夹列表和顺序

settings-header-auto-collapse = 写入后折叠标题（秒）

settings-header-auto-collapse-help = 当正文获得焦点这么多秒并且标头有收件人和主题时，它会折叠到摘要行。 0 将其关闭。

settings-message-badges = 消息徽章

settings-message-badges-help = 选择在打开的邮件标题中显示哪些徽章。优先级标记设置在文件夹和侧边栏下。您还可以右键单击徽章将其隐藏。

settings-reply-from-headers = 选择身份的标头

settings-reply-from-headers-help = 每行一个标头名称，按以下顺序尝试：Delivered-To、X-Original-To、Envelope-To、X-Forwarded-To、Resent-To、To、Cc。未读取的标头不会被读取。

settings-reply-from-headers-invalid = 仅使用这些名称，每行一个：Delivered-To、X-Original-To、Envelope-To、X-Forwarded-To、Resent-To、To、Cc。

settings-reply-from-headers-save = 保存标题顺序

settings-sessions-linked-created = 链接已创建

settings-sessions-linked-details = 链接详情

settings-sessions-linked-kind = 链接类型

settings-sessions-linked-kind-linked = 联动邮箱

settings-sessions-linked-last-used = 上次使用

settings-sessions-linked-line = 链接邮箱访问·来自{ $origin }

settings-sessions-linked-line-hidden = 链接邮箱访问

settings-sessions-linked-origin = 来自

settings-sessions-linked-removed = 未知（已删除帐户）

settings-sessions-linked-revoke = 撤销链接访问权限

settings-sessions-linked-revoke-label = 撤销{ $mailbox }的链接访问权限

settings-sessions-linked-target = 邮箱已打开

settings-sessions-origin-full = 完整地址

settings-sessions-origin-help = 控制此表如何命名链接邮箱会话来自的帐户。只有您在自己的会话中看到它。

settings-sessions-origin-hidden = 隐藏

settings-sessions-origin-masked-local-part = 第一部分被屏蔽的地址

settings-sessions-origin-setting = 显示链接邮箱访问的来源

settings-shortcuts-search = 搜索快捷方式

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
cache-help-durable = 保持最近访问的文件夹和消息随时可以打开。默认情况下，副本保留在此选项卡的内存中。启用无限期存储以在重新加载时保留加密副本。预加载永远不会标记邮件已读或加载远程图像。

cache-keepIndefinitely = 无限期保留缓存的邮件

cache-keepIndefinitely-help = 默认关闭。在重新加载时在此浏览器中保留加密副本，且不会过期。网上消息还在刷新。条目和字节限制仍然适用。关闭此功能或清除缓存的邮件会删除保存的副本。浏览器数据清除也可以将其删除。不会下载附件供离线使用。
