# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-settings. Source: ../en/admin-settings.ftl
admin-settings-key-mail-allow-send = 允许发送
admin-settings-help-mail-allow-send = 允许用户发送邮件。关闭后，编辑器会禁用“发送”及其快捷键，服务器会拒绝提交；仍可保存草稿。
admin-settings-key-mail-allow-reply = 允许回复
admin-settings-help-mail-allow-reply = 允许用户在阅读窗格中及通过快捷键回复和全部回复。关闭后，不再提供这些操作。
admin-settings-key-mail-allow-forward = 允许转发
admin-settings-help-mail-allow-forward = 允许用户以内嵌方式或作为附件转发邮件。关闭后，不再提供这些操作。
admin-settings-key-mail-allow-download = 允许下载
admin-settings-help-mail-allow-download = 允许用户下载邮件和附件。关闭后，网页应用不提供下载，服务器会拒绝邮件导出。
admin-settings-key-mail-allow-print = 允许打印
admin-settings-help-mail-allow-print = 允许用户在阅读窗格中打印邮件。关闭后，不再提供“打印”操作。
admin-settings-key-mail-show-spam = 显示垃圾邮件文件夹
admin-settings-help-mail-show-spam = 显示垃圾邮件（Spam 或 Junk）文件夹和标记为垃圾邮件的邮件。关闭后，它们将在文件夹列表和搜索结果中隐藏；不会删除任何内容。
admin-settings-key-mail-show-auth-status = 显示发件人验证结果
admin-settings-help-mail-show-auth-status = 在阅读窗格中显示 Authentication-Results 标头中的 SPF、DKIM 和 DMARC 结果。
admin-settings-key-mail-close-on-delete = 删除邮件时将其关闭
admin-settings-help-mail-close-on-delete = 当打开的邮件被删除或移至废纸篓时，关闭该邮件所在的标签页或独立窗口。关闭此项后，阅读窗格保持原有行为。
admin-settings-key-links-trusted-senders = 受信任的链接发件人
admin-settings-help-links-trusted-senders = 其已审核邮件中的链接可免于逐封邮件确认直接打开的确切发件人邮箱。每项一个地址。
admin-settings-key-appearance-font-size = 界面字号
admin-settings-help-appearance-font-size = 界面字号，单位为像素，范围 12 至 24。
admin-settings-key-appearance-font-weight = 界面字重
admin-settings-help-appearance-font-weight = 界面文字的基础字重：300、400、500 或 600。强调文字保持其层级。
admin-settings-key-appearance-font-color = 界面文字颜色
admin-settings-help-appearance-font-color = 界面文字颜色，格式为 #RRGGBB。留空则跟随主题。
admin-settings-key-appearance-low-contrast = 减弱文字对比度
admin-settings-help-appearance-low-contrast = 减弱界面文字的对比度。
admin-settings-key-mail-message-list-order = 邮件列表排序
admin-settings-help-mail-message-list-order = 邮件列表的排序：最新、最早、发件人或主题。
admin-settings-key-mail-message-list-style = 邮件列表样式
admin-settings-help-mail-message-list-style = 将邮件列表显示为卡片，或显示为带列标题的行。
admin-settings-key-mail-message-list-preview = 邮件列表预览
admin-settings-help-mail-message-list-preview = 在邮件列表中显示正文摘要。
admin-settings-key-mail-after-delete = 删除邮件之后
admin-settings-help-mail-after-delete = 删除打开的邮件后阅读窗格显示哪封邮件：下一封、上一封或不显示。
admin-settings-key-reader-attachments-at-end = 附件显示在正文之后
admin-settings-help-reader-attachments-at-end = 将附件显示在邮件正文之后，而不是之前。
admin-settings-key-notifications-ask-permission = 提供通知权限请求
admin-settings-help-notifications-ask-permission = 向尚未允许或阻止通知的设备上的用户提供浏览器通知权限提示。

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-title = 设置
admin-settings-description = 适用于整个服务器、各域名和各用户的值。已锁定的值不能被用户或更窄范围的设置更改。
admin-settings-list-caption = 设置
admin-settings-search = 搜索设置
admin-settings-empty = 没有符合搜索条件的设置。
admin-settings-unavailable = 此服务器尚未提供设置模型。
admin-settings-col-setting = 设置
admin-settings-col-value = 值
admin-settings-col-source = 来源
admin-settings-col-overrides = 更窄范围的值
admin-settings-open = 打开
admin-settings-overrides-none = 无
admin-settings-overrides = { $domains } 个域名，{ $users } 个用户
admin-settings-not-found = 没有名为 { $key } 的设置。
admin-settings-value-on = 开
admin-settings-value-off = 关
admin-settings-value-empty = 未设置
admin-settings-value-list = { $count ->
   *[other] { $count } 项
}
admin-settings-detail-key = 键：{ $key }
admin-settings-global = 适用于整个服务器
admin-settings-global-help = 除非域名或用户有更窄范围的值，或在允许的情况下用户自行选择，否则此值适用于所有用户。
admin-settings-value-label = 值
admin-settings-json-hint = JSON。服务器会检查该值，并指出它拒绝的部分。
admin-settings-json-invalid = 这不是有效的 JSON。
admin-settings-user-override = 允许用户更改
admin-settings-user-override-help = 开启时，用户可以在自己的设置中选择自己的值。关闭时，适用此处或其域名设置的值。
admin-settings-lock-help = 锁定后，域名值和用户的选择都会被忽略，此值适用于所有人。
admin-settings-save = 保存
admin-settings-saved = 已保存。将在 30 秒内在每个服务器进程上生效。
admin-settings-saved-restart = 已保存。重启后生效。
admin-settings-dedicated = 此设置请在 { $section } 中更改，该处会先检查其影响。
admin-settings-dedicated-link = 打开 { $section }
admin-settings-pinned = 由环境变量 { $name } 设定。未设置该变量时，将使用此处存储的值。
admin-settings-lock-title = 锁定 { $setting }？
admin-settings-lock-body = 锁定会忽略下方列出的更窄范围的值。这些值会保留，解锁后重新生效。
admin-settings-lock-confirm = 锁定
admin-settings-impact-domain = { $count ->
   *[other] 将忽略 { $count } 个域名值
}
admin-settings-impact-user = { $count ->
   *[other] 将忽略 { $count } 位用户的自选值
}
admin-settings-impact-admin-user = { $count ->
   *[other] 将忽略 { $count } 个单用户值
}
admin-settings-impact-profile = { $count ->
   *[other] 将忽略 { $count } 个功能配置文件值
}
admin-settings-impact-group = { $count ->
   *[other] 将忽略 { $count } 个组值
}
admin-settings-domains = 域名值
admin-settings-domains-help = 域名值会取代整个服务器的值，应用于该域名的用户。
admin-settings-domains-caption = { $setting } 的域名值
admin-settings-domains-empty = 没有域名设有自己的值。
admin-settings-domain = 域名
admin-settings-domain-value = 此域名的值
admin-settings-domain-locked = 对该域名的用户锁定
admin-settings-domain-add = 添加域名值
admin-settings-domain-save = 保存域名值
admin-settings-domain-remove = 移除
admin-settings-domain-remove-title = 移除 { $domain } 的值？
admin-settings-domain-remove-body = { $domain } 的用户将重新使用整个服务器的值。
admin-settings-domain-required = 请输入域名。
admin-settings-locked-yes = 已锁定
admin-settings-locked-no = 未锁定
admin-settings-users = 单用户值
admin-settings-users-help = 管理员为某位用户设置的值。对于限额，用户以地址指定。
admin-settings-users-caption = { $setting } 的单用户值
admin-settings-users-empty = 没有用户拥有管理员设置的值。
admin-settings-user = 用户
admin-settings-user-hint = 账户 ID，限额则填写地址。
admin-settings-user-value = 此用户的值
admin-settings-user-add = 添加单用户值
admin-settings-user-save = 保存用户值
admin-settings-user-required = 请输入用户。
admin-settings-user-remove-title = 移除 { $user } 的值？
admin-settings-user-remove-body = 该用户将使用其域名的值，或整个服务器的值。
admin-settings-source-default = 内置默认值
admin-settings-source-envFallback = 环境变量（无存储值）
admin-settings-source-envPin = 环境变量
admin-settings-source-global = 为整个服务器存储
admin-settings-cat-interface = 界面
admin-settings-cat-appearance = 外观
admin-settings-cat-mail = 邮件
admin-settings-cat-folders = 文件夹布局
admin-settings-cat-notifications = 通知
admin-settings-cat-offline = 离线使用
admin-settings-cat-assistant = AI 助手
admin-settings-cat-upstream = 邮件服务器
admin-settings-cat-signin = 登录
admin-settings-cat-admin = 管理控制台
admin-settings-cat-passwords = 密码
admin-settings-cat-limits = 限额与用量
admin-settings-cat-plugins = 插件
admin-settings-cat-features = 功能权限
admin-settings-cat-integrations = 集成权限
admin-settings-key-cap-integration-nextcloud = Nextcloud
admin-settings-help-cap-integration-nextcloud = 允许与账户关联的 Nextcloud 文件、共享以及日历/联系人/任务操作。
admin-settings-key-cap-integration-google = Google 桥接
admin-settings-help-cap-integration-google = 允许 Google 桥接操作，包括获取和刷新令牌。
admin-settings-key-cap-integration-microsoft = Microsoft 桥接
admin-settings-help-cap-integration-microsoft = 允许 Microsoft 桥接操作，包括获取和刷新令牌。
admin-settings-key-cap-integration-family = 单项集成
admin-settings-help-cap-integration-family = 针对单个集成标识符的权限；运行时覆盖范围见功能目录。
admin-settings-key-cap-ui-plugin-family = 单个界面插件
admin-settings-help-cap-ui-plugin-family = 针对某个界面插件的权限。请使用集成目录中的确切策略键。
admin-settings-key-cap-ai-assistant = AI 助手
admin-settings-help-cap-ai-assistant = 允许配置和调用助手。
admin-settings-key-cap-plugins = 插件
admin-settings-help-cap-plugins = 允许界面插件和引擎插件的操作。
admin-settings-key-cap-pim-calendar = 日历
admin-settings-help-cap-pim-calendar = 允许日历方法和订阅。
admin-settings-key-cap-pim-contacts = 联系人
admin-settings-help-cap-pim-contacts = 允许联系人和通讯录方法。
admin-settings-key-cap-pim-tasks = 任务
admin-settings-help-cap-pim-tasks = 允许任务方法。
admin-settings-key-cap-pim-notes = 笔记
admin-settings-help-cap-pim-notes = 允许笔记方法。
admin-settings-key-cap-export = 导出
admin-settings-help-cap-export = 允许导出邮件和 PIM 数据。
admin-settings-key-cap-import = 导入
admin-settings-help-cap-import = 允许导入邮件和 PIM 数据。
admin-settings-key-cap-api-keys = API 密钥
admin-settings-help-cap-api-keys = API 密钥策略。执行覆盖范围请查看功能目录。
admin-settings-key-cap-oauth-apps = OAuth 应用
admin-settings-help-cap-oauth-apps = OAuth 应用策略。执行覆盖范围请查看功能目录。
admin-settings-key-cap-rules = 邮件规则
admin-settings-help-cap-rules = 允许受管理的邮件规则和自动回复方法。
admin-settings-key-cap-rules-forward = 规则转发
admin-settings-help-cap-rules-forward = 转发策略。执行覆盖范围请查看功能目录。
admin-settings-key-cap-remote-images = 远程图片
admin-settings-help-cap-remote-images = 远程图片能力。现有的隐私设置仍然管控请求；请查看功能覆盖范围。
admin-settings-key-cap-user-background = 自定义背景
admin-settings-help-cap-user-background = 自定义背景能力，默认关闭。请查看功能覆盖范围。
admin-settings-key-cap-user-css = 自定义 CSS
admin-settings-help-cap-user-css = 自定义 CSS 能力，默认关闭。请查看功能覆盖范围。
admin-settings-key-limit-attachment-bytes = 附件大小限制
admin-settings-help-limit-attachment-bytes = 附件策略，以字节为单位，默认 50000000。运行时执行尚待实现。
admin-settings-key-limit-send-per-hour = 每小时发送限制
admin-settings-help-limit-send-per-hour = 每小时的提交策略；0 表示不限制。运行时执行尚待实现。
admin-settings-key-limit-recipients-per-message = 每封邮件的收件人数
admin-settings-help-limit-recipients-per-message = 收件人数策略；0 表示不限制。运行时执行尚待实现。
admin-settings-cat-import = 导入
admin-settings-cat-webhooks = Webhook
admin-settings-cat-realtime = 实时更新
admin-settings-cat-privacy = 隐私
admin-settings-cat-observability = 可观测性
admin-settings-cat-egress = 出站连接
admin-settings-cat-updates = 更新
admin-settings-cat-backups = 备份
admin-settings-cat-about = 关于
admin-settings-cat-config = 配置
admin-settings-cat-jobs = 任务
admin-settings-cat-maintenance = 维护
admin-settings-cat-health = 健康检查
admin-settings-cat-diagnostics = 诊断
admin-settings-cat-users = 用户
admin-settings-cat-server-text = 服务器文本
admin-settings-key-ui-language = 语言
admin-settings-help-ui-language = 尚未选择语言的用户所使用的邮件客户端语言。“与浏览器相同”会跟随各浏览器的语言。
admin-settings-key-ui-notify-new-version = 新版本提示
admin-settings-help-ui-notify-new-version = 开启（默认）时，若网页应用在用户加载页面后部署了更新版本，会向已登录用户显示带有“立即重新加载”和“稍后”的提示。关闭则不显示；用户将在下次重新加载时获得新代码。
admin-settings-key-ui-undo-history-size = 撤销与重做历史大小
admin-settings-help-ui-undo-history-size = 撤销和重做按钮记住的近期邮件操作数量（归档、删除、移动、标为已读、标记、标签等），范围 0 到 50。0 会关闭这些按钮和 Ctrl+Z 快捷键。默认为 50。用户可以选择自己的大小，但不能超过下方的上限，除非管理员将其锁定。
admin-settings-key-folders-reorder-enabled = 文件夹拖放
admin-settings-help-folders-reorder-enabled = 允许用户在侧边栏和设置中拖动文件夹来重新排列。关闭会隐藏拖动手柄和移动命令。除非此项被锁定，用户可以为自己关闭。
admin-settings-key-folders-reorder-allow-nesting = 文件夹嵌套
admin-settings-help-folders-reorder-allow-nesting = 允许用户将文件夹拖放到另一个文件夹上以嵌套，或拖回顶层。关闭则每个文件夹保持在当前父级之下，只允许在同级之间重新排序。
admin-settings-key-folders-reorder-allow-server-move = 在服务器上移动文件夹
admin-settings-help-folders-reorder-allow-server-move = 允许嵌套或取消嵌套时更改文件夹在邮件服务器上的真实父级，因此在所有位置都会改变。关闭则这些移动仅作为此应用中的排列保留。
admin-settings-key-folders-reorder-allow-system-folders = 移动系统文件夹
admin-settings-help-folders-reorder-allow-system-folders = 允许用户拖动收件箱、草稿、已发送、垃圾邮件、废纸篓和归档来重新排序。关闭则它们固定在顶部。它们只在此应用中重新排序，不会在服务器上重命名或移动。
admin-settings-key-folders-reorder-max-depth = 文件夹嵌套深度
admin-settings-help-folders-reorder-max-depth = 一次拖动可创建的文件夹最大嵌套层级，范围 1 到 16。会使某个文件夹或其子文件夹超出该深度的移动将被拒绝。
admin-settings-key-folders-order-locked = 固定文件夹顺序
admin-settings-help-folders-order-locked = 为范围内所有人固定文件夹顺序。拖动和移动命令将关闭，用户会看到锁定提示。固定项和管理员设置的顺序保持不变。
admin-settings-key-folders-pinned-admin = 强制固定的文件夹
admin-settings-help-folders-pinned-admin = 显示在每位用户侧边栏顶部“已固定”分组中的文件夹。每行一项，形式为 role:inbox 或文件夹路径，如 Shared/Support。用户无法取消固定或移动它们。没有对应文件夹的条目会被跳过。
admin-settings-key-folders-order-default = 默认文件夹顺序
admin-settings-help-folders-order-default = 尚未自行排列的用户所用的初始文件夹顺序，从前到后。每行一项，形式为 role:<role> 或文件夹路径。未列出的文件夹按服务器的顺序排在后面。用户自己的顺序会取代此顺序。
admin-settings-key-folders-new-folder-placement = 新文件夹位置
admin-settings-help-folders-new-folder-placement = 服务器上新出现的文件夹在同级中的放置位置：末尾，或按字母顺序。
admin-settings-key-ui-undo-history-max = 撤销与重做历史上限
admin-settings-help-ui-undo-history-max = 任何用户在撤销和重做历史中最多可保留的操作数，范围 0 到 50。用户自己的大小会被降低到此值。0 会为所有人关闭历史。历史保存在浏览器标签页中，退出登录时清除。
admin-settings-key-ui-bar-mail-top-bar = 邮件工具栏
admin-settings-help-ui-bar-mail-top-bar = “pinned”会始终显示带有布局控件的邮件工具栏。“auto”会将其隐藏在一个细条后面，用户指向、点按或用 Tab 键移到该处时展开。
admin-settings-key-ui-bar-workspace-tabs = 工作区标签页
admin-settings-help-ui-bar-workspace-tabs = “pinned”会始终显示工作区标签栏。“auto”只显示带有当前标签页名称的细条，直到用户将其打开。
admin-settings-key-ui-bar-mail-ribbon = 邮件功能区
admin-settings-help-ui-bar-mail-ribbon = 适用于功能区布局。“pinned”保持功能区展开。“auto”只显示标签页名称；打开某个标签页时，功能区会覆盖在邮件列表之上。
admin-settings-key-ui-bar-composer-ribbon = 编辑器功能区
admin-settings-help-ui-bar-composer-ribbon = 适用于所有撰写窗口。“pinned”保持编辑器功能区展开。“auto”只显示标签页名称；打开标签页时会将邮件正文下推，而不是覆盖它。
admin-settings-key-appearance-theme = 主题
admin-settings-help-appearance-theme = 用户在自行选择之前看到的颜色主题。锁定后，所有用户都使用此主题。
admin-settings-key-appearance-accent = 强调色
admin-settings-help-appearance-accent = 主题的强调色。留空表示使用主题自带的强调色。
admin-settings-key-appearance-badge-color = 未读徽章颜色
admin-settings-help-appearance-badge-color = 未读数徽章的颜色。默认保持主题颜色；Accent 跟随强调色；accent-lighter 和 accent-darker 使用比强调色亮或暗 20% 的颜色，数字为黑色或白色。锁定会覆盖账户的选择。
admin-settings-key-appearance-density = 密度
admin-settings-help-appearance-density = 列表和控件所占的空间大小：紧凑、适中或宽松。
admin-settings-key-appearance-font = 界面字体
admin-settings-key-packages-allowed = 允许包裹追踪
admin-settings-help-packages-allowed = 允许受影响的账户使用包裹追踪。还必须在包裹追踪中启用服务提供方，并且每位用户都需选择启用。停用此设置会立即阻止新的提供方调用。
admin-settings-help-appearance-font = 为没有字体偏好的账户选择内置字体，或为所有受影响的账户锁定该选择。字体从此服务器加载，不会向 Google Fonts 发出请求。默认使用主题字体。
admin-settings-key-appearance-brand-name = 产品名称
admin-settings-help-appearance-brand-name = 显示在登录页面和浏览器标签页中的名称。
admin-settings-key-login-notice = 登录提示
admin-settings-help-login-notice = 显示在登录页面上的简短文字，例如支持联系方式。留空表示没有。
admin-settings-key-mail-read-receipts = 已读回执
admin-settings-key-receipts-dsn-allowed = 投递状态请求
admin-settings-help-receipts-dsn-allowed = 允许账户请求 SMTP 投递状态通知。这不会请求或证明邮件已被阅读。
admin-settings-help-mail-read-receipts = 打开的邮件请求已读回执时的处理方式：询问用户、从不发送，或总是发送。
admin-settings-key-notifications-enabled = 新邮件通知
admin-settings-help-notifications-enabled = 用户是否收到新邮件通知。这不会改变他们的通知规则。
admin-settings-key-offline-retention-days = 离线缓存保留时长
admin-settings-help-offline-retention-days = 设备为离线使用保留邮件的天数。0 表示没有时长限制。
admin-settings-key-offline-budget-mb = 离线缓存大小
admin-settings-help-offline-budget-mb = 设备可用于离线邮件的兆字节数。0 表示没有离线缓存。
admin-settings-key-assistant-enabled = AI 助手
admin-settings-help-assistant-enabled = 用户是否能看到 AI 助手。AI 助手中的部署开关仍然优先于此项。
admin-settings-key-assist-profile = AI 提供方配置文件
admin-settings-help-assist-profile = 为这些用户提供服务的提供方配置文件 ID（AI 助手 > 提供方配置文件）。留空则使用部署端点。不存在的配置文件 ID 会为这些用户关闭 AI 助手。
admin-settings-key-upstream-default = 默认邮件服务器
admin-settings-help-upstream-default = 当没有域名指定邮件服务器时，地址登录所用的邮件服务器。域名表优先于此项。
admin-settings-key-upstream-proxy-timeout-secs = 邮件服务器超时
admin-settings-help-upstream-proxy-timeout-secs = 向邮件服务器发出的一次请求最多可花费的时间，单位为秒。
admin-settings-key-signin-new-users = 新用户
admin-settings-help-signin-new-users = 谁可以在首次登录时创建账户：任何人（open）、仅限事先添加的地址（preprovisioned）、这些加上已导入的用户（provisioned），或不允许新用户（closed）。现有账户不受影响。在登录策略中更改。
admin-settings-key-signin-new-users-mail = 密码登录：新用户
admin-settings-help-signin-new-users-mail = 新邮箱用户使用密码登录（IMAP、POP3、Exchange）的规则。“继承”使用“新用户”。
admin-settings-key-signin-new-users-sso = 单点登录：新用户
admin-settings-help-signin-new-users-sso = 新用户通过单点登录或受信任标头登录的规则。“继承”使用“新用户”。
admin-settings-key-signin-new-users-linked = 关联邮箱：新用户
admin-settings-help-signin-new-users-linked = 关联一个在此尚无账户的邮箱的规则，按被关联邮箱自身的域名判断。“继承”使用“新用户”。
admin-settings-key-signin-new-users-linked-imap = 关联 IMAP 邮箱：新用户
admin-settings-help-signin-new-users-linked-imap = 仅适用于关联 IMAP 邮箱的规则。“继承”使用“关联邮箱”。
admin-settings-key-signin-new-users-linked-pop3 = 关联 POP3 邮箱：新用户
admin-settings-help-signin-new-users-linked-pop3 = 仅适用于关联 POP3 邮箱的规则。“继承”使用“关联邮箱”。
admin-settings-key-signin-new-users-linked-ews = 关联 Exchange 邮箱：新用户
admin-settings-help-signin-new-users-linked-ews = 仅适用于关联 Exchange（EWS）邮箱的规则。“继承”使用“关联邮箱”。
admin-settings-key-signin-preprovision-ttl-days = 预添加用户有效期（天）
admin-settings-help-signin-preprovision-ttl-days = 未给出其他有效期时，事先添加的待处理用户的有效时长。
admin-settings-key-signin-preprovision-max-ttl-days = 预添加用户最长有效期（天）
admin-settings-help-signin-preprovision-max-ttl-days = 管理员可为事先添加的待处理用户设置的最长有效期。
admin-settings-key-signin-preprovision-domain-wide = 允许整个域名的预添加条目
admin-settings-help-signin-preprovision-domain-wide = 允许放行某个域名下所有人的条目，并设有使用次数上限。默认关闭。
admin-settings-key-signin-preprovision-retention-days = 保留已过期的待处理用户（天）
admin-settings-help-signin-preprovision-retention-days = 已过期的待处理用户在被移除前保留的时长。
admin-settings-key-signin-domain-mode = 登录域名
admin-settings-help-signin-domain-mode = 是任何域名都可登录，还是仅限列出的域名。在登录策略中更改。
admin-settings-key-signin-refusal-detail = 拒绝消息
admin-settings-help-signin-refusal-detail = 被拒绝的登录是否说明原因（账户已停用、地址不被允许），还是与密码错误给出相同的回答。在登录策略中更改。
admin-settings-key-signin-ldap-bind-precheck = 登录前的 LDAP 检查
admin-settings-help-signin-ldap-bind-precheck = 在邮件服务器之前先对照目录检查密码。在测试绑定后，于集成中更改。
admin-settings-key-twofa-pending-ttl-secs = 第二因素时限
admin-settings-help-twofa-pending-ttl-secs = 用户在输入密码后输入第二因素的时限，单位为秒。
admin-settings-key-twofa-max-attempts = 第二因素尝试次数
admin-settings-help-twofa-max-attempts = 第二因素验证码错误多少次后登录将重新开始。
admin-settings-key-admin-session-idle-secs = 控制台空闲超时
admin-settings-help-admin-session-idle-secs = 管理控制台会话在没有请求的情况下持续的时长，单位为秒。
admin-settings-key-admin-session-absolute-secs = 控制台会话上限
admin-settings-help-admin-session-absolute-secs = 管理控制台会话最长持续的时间，无论是否活跃，单位为秒。
admin-settings-key-admin-session-rotate-secs = 控制台会话 ID 轮换
admin-settings-help-admin-session-rotate-secs = 管理控制台在使用期间更换会话 ID 的频率，单位为秒（0 = 仅在二次确认后更换）。轮换不会延长空闲或绝对时限。
admin-settings-key-passwd-policy-min-length = 密码最小长度
admin-settings-help-passwd-policy-min-length = 新密码至少应有的字符数。
admin-settings-key-passwd-policy-require-upper = 要求大写字母
admin-settings-help-passwd-policy-require-upper = 新密码必须包含大写字母。
admin-settings-key-passwd-policy-require-lower = 要求小写字母
admin-settings-help-passwd-policy-require-lower = 新密码必须包含小写字母。
admin-settings-key-passwd-policy-require-digit = 要求数字
admin-settings-help-passwd-policy-require-digit = 新密码必须包含数字。
admin-settings-key-passwd-policy-require-symbol = 要求符号
admin-settings-help-passwd-policy-require-symbol = 新密码必须包含既不是字母也不是数字的字符。
admin-settings-key-limit-storage-bytes = 存储限制
admin-settings-help-limit-storage-bytes = 一位用户在此服务器上最多可存储的字节数。0 表示不限制。
admin-settings-key-limit-send-per-day = 每日发送邮件数
admin-settings-help-limit-send-per-day = 一位用户每天最多可发送的邮件数。0 表示不限制。
admin-settings-key-limit-ai-calls-per-day = 每日 AI 助手调用次数
admin-settings-help-limit-ai-calls-per-day = 一位用户每天最多可发出的 AI 助手请求数。0 表示不限制。
admin-settings-key-quota-warn-thresholds = 用量警告级别
admin-settings-help-quota-warn-thresholds = 达到限额的多少百分比时向用户发出警告，从低到高。
admin-settings-key-quota-warn-enabled = 用量警告
admin-settings-help-quota-warn-enabled = 用户接近限额时是否收到警告。
admin-settings-key-usage-retention-days = 用量历史
admin-settings-help-usage-retention-days = 每日用量计数保留的天数。
admin-settings-key-usage-meter-api = 统计 API 使用
admin-settings-help-usage-meter-api = API 请求是否计入用量数据。
admin-settings-key-usage-measure-interval-hours = 存储测量间隔
admin-settings-help-usage-measure-interval-hours = 测量每位用户所存数据的频率，单位为小时。
admin-settings-key-assistant-rate-limit-per-min = AI 助手速率限制
admin-settings-help-assistant-rate-limit-per-min = 整个服务器每分钟最多的 AI 助手请求数。0 表示不限制。
admin-settings-key-plugins-kv-max-key-bytes = 插件键大小
admin-settings-help-plugins-kv-max-key-bytes = 插件可存储的最长键，单位为字节。
admin-settings-key-plugins-kv-max-value-bytes = 插件值大小
admin-settings-help-plugins-kv-max-value-bytes = 插件在一个键下可存储的最大值，单位为字节。
admin-settings-key-plugins-kv-max-total-bytes = 插件存储
admin-settings-help-plugins-kv-max-total-bytes = 一个插件最多可存储的总字节数。
admin-settings-key-plugins-kv-max-keys = 插件键数量
admin-settings-help-plugins-kv-max-keys = 一个插件最多可存储的键数量。
admin-settings-key-import-max-messages = 导入邮件数量
admin-settings-help-import-max-messages = 一次导入最多可带入的邮件数。
admin-settings-key-import-max-message-bytes = 导入邮件大小
admin-settings-help-import-max-message-bytes = 导入可接受的最大邮件，单位为字节。
admin-settings-key-webhooks-max-attempts = Webhook 尝试次数
admin-settings-help-webhooks-max-attempts = Webhook 投递在放弃之前尝试的次数。
admin-settings-key-webhooks-timeout-secs = Webhook 超时
admin-settings-help-webhooks-timeout-secs = 一次 Webhook 投递最多可花费的时间，单位为秒。
admin-settings-key-outbox-max-attempts = 发送尝试次数
admin-settings-help-outbox-max-attempts = 邮件在被报告为未发送之前，交给邮件服务器的次数。
admin-settings-key-realtime-recheck-secs = 实时更新复查
admin-settings-help-realtime-recheck-secs = 已打开的实时更新连接检查其会话是否仍然有效的频率，单位为秒。
admin-settings-key-privacy-watermark-enabled = 阅读者水印
admin-settings-help-privacy-watermark-enabled = 在已打开的邮件上显示带有阅读者地址的水印。
admin-settings-key-privacy-watermark-opacity = 水印强度
admin-settings-help-privacy-watermark-opacity = 水印的可见程度，从 0.02（淡）到 0.5。
admin-settings-key-privacy-open-tracking-retention-days = 打开追踪历史
admin-settings-help-privacy-open-tracking-retention-days = 阅读追踪记录保留的天数。
admin-settings-key-privacy-open-tracking-unbound-ttl-hours = 未发送的追踪令牌
admin-settings-help-privacy-open-tracking-unbound-ttl-hours = 追踪令牌在被删除前等待其邮件发送的小时数。
admin-settings-key-masked-email-domain = 隐藏地址域名

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-masked-email-domain = 隐藏地址所创建在的域名。留空则关闭隐藏地址。
admin-settings-key-abuse-report-address = 滥用举报地址
admin-settings-help-abuse-report-address = 垃圾邮件和滥用举报的发送目标。
admin-settings-key-dlp-admin-address = 数据泄露通知
admin-settings-help-dlp-admin-address = 数据防泄漏通知的发送目标。
admin-settings-key-observability-error-forward-url = 错误转发地址
admin-settings-help-observability-error-forward-url = 客户端错误报告转发到的 https 地址。留空表示不转发。
admin-settings-key-egress-probe-url = 出站测试地址
admin-settings-help-egress-probe-url = 用于测试出站代理的 https 地址。
admin-settings-key-push-vapid-contact = 推送联系人
admin-settings-help-push-vapid-contact = 每次通知时提供给推送服务的联系方式（mailto: 或 https:）。
admin-settings-key-push-quiet-hours = 推送静默时段
admin-settings-help-push-quiet-hours = 不发送推送通知的时段（UTC）。留空表示没有。
admin-settings-key-updates-check = 检查更新
admin-settings-help-updates-check = 服务器是否每天检查一次是否有新版本，并在仪表板上显示。
admin-settings-key-updates-feed-url = 发布源
admin-settings-help-updates-feed-url = 更新检查所读取的 https 地址。
admin-settings-key-updates-apply = 应用更新
admin-settings-help-updates-apply = 是否可以通过主机更新代理在控制台中启动更新。
admin-settings-key-backup-retention-count = 保留的备份数
admin-settings-help-backup-retention-count = 保留的数据库备份数量。
admin-settings-key-about-security-contact = 安全联系人
admin-settings-help-about-security-contact = security.txt 中的联系方式（mailto: 或 https:）。
admin-settings-key-about-security-policy-url = 安全策略
admin-settings-help-about-security-policy-url = 你的安全策略的地址，列在 security.txt 中。
admin-settings-key-about-project-links = 关于链接
admin-settings-help-about-project-links = 显示在“关于”页面上的链接，每个都有标签和 https 地址。
admin-settings-key-config-snapshots-kept = 配置快照
admin-settings-help-config-snapshots-kept = 保留的配置快照数量。
admin-settings-key-jobs-history-days = 任务历史
admin-settings-help-jobs-history-days = 任务运行记录保留的天数。
admin-settings-key-maintenance-announce-minutes-default = 维护通知提前时间
admin-settings-help-maintenance-announce-minutes-default = 在维护窗口开始前多少分钟告知用户。
admin-settings-key-maintenance-default-message = 维护消息
admin-settings-help-maintenance-default-message = 维护期间用户看到的文字，按语言设置。
admin-settings-key-health-disk-free-min-percent = 磁盘可用空间下限
admin-settings-help-health-disk-free-min-percent = 当可用磁盘空间低于此百分比时，就绪检查失败。
admin-settings-key-observability-otlp-metrics = 通过 OTLP 发送指标
admin-settings-help-observability-otlp-metrics = 将服务器指标发送到启动时设定的 OTLP 收集器。
admin-settings-key-metrics-drilldown-top-n = 指标明细行数
admin-settings-help-metrics-drilldown-top-n = 指标细分显示的行数。
admin-settings-key-metrics-rollup-days = 指标历史
admin-settings-help-metrics-rollup-days = 保留每小时指标摘要的天数。
admin-settings-key-observability-client-timing-enabled = 浏览器加载时间
admin-settings-help-observability-client-timing-enabled = 以匿名汇总直方图的形式，收集已登录浏览器的页面加载耗时。不会识别任何人、会话或页面。关闭即可停止收集。
admin-settings-key-observability-client-timing-sample-rate-percent = 浏览器加载时间采样率
admin-settings-help-observability-client-timing-sample-rate-percent = 上报加载耗时的浏览器会话百分比（1 到 100）。
admin-settings-key-observability-client-timing-min-samples = 加载时间最少样本数
admin-settings-help-observability-client-timing-min-samples = 在某个时段至少有这么多样本之前，加载时间的中位数和第 95 百分位数保持隐藏。
admin-settings-key-observability-client-timing-folder-loads = 文件夹加载时间
admin-settings-help-observability-client-timing-folder-loads = 测量文件夹显示邮件列表所需的时间，并按来自浏览器缓存还是网络分开统计。默认开启。除非此设置被锁定，用户可以为自己关闭。需要开启“浏览器加载时间”。
admin-settings-key-observability-client-timing-message-loads = 邮件加载时间
admin-settings-help-observability-client-timing-message-loads = 测量邮件显示邮件头和已净化正文所需的时间，并按来自缓存、预取还是网络分开统计。默认开启。除非此设置被锁定，用户可以为自己关闭。需要开启“浏览器加载时间”。
admin-settings-key-diagnostics-retention-days = 诊断历史
admin-settings-help-diagnostics-retention-days = 诊断运行记录保留的天数。
admin-settings-key-data-export-link-hours = 数据导出下载期限
admin-settings-help-data-export-link-hours = 数据导出在被删除前可供下载的小时数。
admin-settings-key-dsr-enabled = 数据请求
admin-settings-help-dsr-enabled = 用户是否可以在设置中提交数据请求（访问、下载、更正、删除、限制、反对）。
admin-settings-key-dsr-type-access = 接受访问请求
admin-settings-help-dsr-type-access = 是否向用户提供访问请求类型。
admin-settings-key-dsr-type-portability = 接受下载（可携带副本）请求
admin-settings-help-dsr-type-portability = 是否向用户提供下载（可携带副本）请求类型。
admin-settings-key-dsr-type-rectification = 接受更正请求
admin-settings-help-dsr-type-rectification = 是否向用户提供更正请求类型。
admin-settings-key-dsr-type-erasure = 接受删除请求
admin-settings-help-dsr-type-erasure = 是否向用户提供删除请求类型。
admin-settings-key-dsr-type-restriction = 接受限制请求
admin-settings-help-dsr-type-restriction = 是否向用户提供限制请求类型。
admin-settings-key-dsr-type-objection = 接受反对请求
admin-settings-help-dsr-type-objection = 是否向用户提供反对请求类型。
admin-settings-key-dsr-type-other = 接受其他请求
admin-settings-help-dsr-type-other = 是否向用户提供通用请求类型。
admin-settings-key-dsr-approval-export = 访问和下载请求的审批
admin-settings-help-dsr-approval-export = auto 在请求确认后立即开始导出。admin 则等待管理员批准。
admin-settings-key-dsr-approval-erasure = 删除请求的审批
admin-settings-help-dsr-approval-erasure = auto 在请求确认后立即安排删除。admin 则等待管理员批准。
admin-settings-key-dsr-approval-other = 其他请求的处理
admin-settings-help-dsr-approval-other = auto 在请求确认后将其标记为处理中。admin 则等待管理员批准。
admin-settings-key-dsr-reauth-required = 请求的重新验证
admin-settings-help-dsr-reauth-required = 提交导出或删除请求以及创建下载链接时，要求重新输入密码或第二因素。
admin-settings-key-dsr-confirmation-required = 确认消息
admin-settings-help-dsr-confirmation-required = 向请求者的收件箱发送带验证码的确认消息；在输入验证码之前，请求不会继续。
admin-settings-key-dsr-identity-verification-required = 身份核实步骤
admin-settings-help-dsr-identity-verification-required = 在批准请求之前，管理员必须记录已核实请求者的身份。
admin-settings-key-dsr-erasure-grace-days = 删除宽限期
admin-settings-help-dsr-erasure-grace-days = 从批准到删除之间的天数。请求者可在此期间取消。0 表示在下一次工作进程运行时执行删除。
admin-settings-key-dsr-sla-days = 答复请求的天数
admin-settings-help-dsr-sla-days = 从提交到向用户和管理员显示的到期日之间的天数。
admin-settings-key-dsr-sla-max-extension-days = 最长延期
admin-settings-help-dsr-sla-max-extension-days = 管理员在记录理由后可为请求的到期日增加的最多天数。
admin-settings-key-dsr-sla-reminder-days = 到期日标记
admin-settings-help-dsr-sla-reminder-days = 在到期日前多少天，请求在队列中被标记为即将到期。
admin-settings-key-dsr-export-mail-allowed = 导出中的邮件
admin-settings-help-dsr-export-mail-allowed = 允许请求者在导出中包含邮件。只包含保存在此服务器自身缓存中的邮件；仍留在上游邮件服务器上的邮件不包含在内。
admin-settings-key-dsr-export-mail-format = 导出中的邮件格式
admin-settings-help-dsr-export-mail-format = mbox 每个文件夹写入一个文件；eml 每封邮件写入一个文件。
admin-settings-key-dsr-export-include-settings = 导出：设置
admin-settings-help-dsr-export-include-settings = 在导出中包含设置。
admin-settings-key-dsr-export-include-identities = 导出：身份
admin-settings-help-dsr-export-include-identities = 在导出中包含身份。
admin-settings-key-dsr-export-include-contacts = 导出：联系人
admin-settings-help-dsr-export-include-contacts = 在导出中包含联系人。
admin-settings-key-dsr-export-include-calendars = 导出：日历
admin-settings-help-dsr-export-include-calendars = 在导出中包含日历。
admin-settings-key-dsr-export-include-notes = 导出：笔记
admin-settings-help-dsr-export-include-notes = 在导出中包含笔记。
admin-settings-key-dsr-export-include-rules = 导出：过滤器和规则
admin-settings-help-dsr-export-include-rules = 在导出中包含过滤器和规则。
admin-settings-key-dsr-export-include-sessions = 导出：会话、设备和登录历史
admin-settings-help-dsr-export-include-sessions = 在导出中包含会话、设备和登录历史。
admin-settings-key-dsr-export-include-consent = 导出：同意记录
admin-settings-help-dsr-export-include-consent = 在导出中包含同意记录。
admin-settings-key-dsr-export-include-audit = 导出：与该人相关的审计条目
admin-settings-help-dsr-export-include-audit = 在导出中包含与该人相关的审计条目。
admin-settings-key-dsr-export-max-megabytes = 导出大小上限
admin-settings-help-dsr-export-max-megabytes = 导出的最大兆字节数。会超出上限的部分将被省略，并列在清单中。
admin-settings-key-dsr-export-format = 导出格式
admin-settings-help-dsr-export-format = json 写入 JSON 文件。jsonCsv 还会为列表数据写入 CSV 文件。
admin-settings-key-dsr-export-encrypt = 静态加密导出
admin-settings-help-dsr-export-encrypt = 已生成的导出在被下载或过期之前，使用服务器密钥加密存储。
admin-settings-key-dsr-download-link-minutes = 下载链接有效期
admin-settings-help-dsr-download-link-minutes = 下载链接创建后可用的分钟数。导出本身按“数据导出下载期限”保留。
admin-settings-key-dsr-download-single-use = 一次性下载链接
admin-settings-help-dsr-download-single-use = 下载链接在使用一次后失效。
admin-settings-key-dsr-retention-completed-days = 已完成请求的保留期
admin-settings-help-dsr-retention-completed-days = 已完成的请求及其记录在被删除前保留的天数。
admin-settings-key-dsr-notify-admins = 为管理员记录新请求
admin-settings-help-dsr-notify-admins = 将每个新请求作为安全事件记录到审计日志中，管理员和告警接收端都能看到。
admin-settings-key-dsr-notify-requester = 向请求者发送进度消息
admin-settings-help-dsr-notify-requester = 请求状态变化时，在请求者的收件箱中放入一条消息。
admin-settings-key-dsr-contact-name = 数据保护联系人姓名
admin-settings-help-dsr-contact-name = 在数据请求页面向用户显示的姓名。
admin-settings-key-dsr-contact-email = 数据保护联系人地址
admin-settings-help-dsr-contact-email = 在数据请求页面向用户显示的电子邮件地址。
admin-settings-key-dsr-contact-url = 数据保护联系页面
admin-settings-help-dsr-contact-url = 在数据请求页面向用户显示的链接。
admin-settings-key-dsr-on-behalf-allowed = 代用户提交请求
admin-settings-help-dsr-on-behalf-allowed = 允许管理员代用户提交请求，例如通过邮寄收到的请求。
admin-settings-key-dsr-rate-limit-per-day = 每人每天的请求数
admin-settings-help-dsr-rate-limit-per-day = 一个人在 24 小时内最多可提交的请求数。
admin-settings-key-dsr-legal-hold-subjects = 法律保留
admin-settings-help-dsr-legal-hold-subjects = 禁止删除的地址或 @domain 条目。针对它们的删除请求会作为已阻止留在队列中，直到该条目被移除。
admin-settings-key-scrub-enabled = 清除个人信息
admin-settings-help-scrub-enabled = 允许“清除个人信息”操作，以及可在删除之后执行的清除。关闭后两者都不会运行。
admin-settings-key-scrub-pseudonym-algorithm = 假名长度
admin-settings-help-scrub-pseudonym-algorithm = 替代标识符的带密钥假名的生成方式：在当前密钥代下使用 HMAC-SHA256，截取为 16 或 32 个十六进制字符。
admin-settings-key-scrub-include-logs = 清除日志文件
admin-settings-help-scrub-include-logs = 清除时也会改写已轮转日志文件（包括压缩文件）中的标识符。
admin-settings-key-scrub-auto-run-on-erasure = 删除后清除
admin-settings-help-scrub-auto-run-on-erasure = 删除完成后，还会将审计历史和日志中该人的标识符替换为假名。默认关闭。
admin-settings-key-scrub-key-retention-days = 假名密钥保留期
admin-settings-help-scrub-key-retention-days = 假名密钥轮换下线后保留的天数。0 表示一直保留。删除之后，旧的假名将无法再重新计算，用该密钥生成的墓碑记录也无法再重放。
admin-settings-key-backup-propagate-erasure = 恢复时重放删除
admin-settings-help-backup-propagate-erasure = 恢复时会重放比备份更新的删除墓碑记录，使备份之后被删除的人再次被移除。
admin-settings-key-backup-tombstone-retention-days = 删除墓碑记录保留期
admin-settings-help-backup-tombstone-retention-days = 删除墓碑记录保留的天数。0 表示永久保留。
admin-settings-key-dsr-text-intro = 数据请求简介
admin-settings-help-dsr-text-intro = 显示在请求表单上方的文字。每种语言一个值。
admin-settings-key-dsr-text-confirmation = 确认消息文字
admin-settings-help-dsr-text-confirmation = 添加到确认消息中的文字。每种语言一个值。
admin-settings-key-dsr-text-completion = 完成消息文字
admin-settings-help-dsr-text-completion = 添加到请求完成时所发消息中的文字。每种语言一个值。
admin-settings-key-server-text-family = 服务器文本
admin-settings-help-server-text-family = 服务器写入邮件和通知的文字，按语言和域名设置。
admin-settings-cat-sieve = 邮件过滤器
admin-settings-key-sieve-max-script-bytes = 过滤脚本大小上限
admin-settings-help-sieve-max-script-bytes = 单个 Sieve 过滤脚本的最大字节数。
admin-settings-key-sieve-max-scripts = 过滤脚本数量上限
admin-settings-help-sieve-max-scripts = 每个账户可保存的脚本最大数量。
admin-settings-key-sieve-max-redirects = 重定向数量上限
admin-settings-help-sieve-max-redirects = 一个脚本中重定向命令的最大数量。
admin-settings-key-sieve-vacation-allowed = 休假过滤操作
admin-settings-help-sieve-vacation-allowed = 在后端支持时，允许脚本发送休假回复。
admin-settings-key-sieve-allowed-extensions = 允许的过滤扩展
admin-settings-help-sieve-allowed-extensions = 可选的小写 Sieve 扩展名列表。后端也必须支持每个扩展。
admin-settings-key-cap-sieve-raw-edit = 编辑过滤脚本源码
admin-settings-help-cap-sieve-raw-edit = 允许所选账户直接编辑 Sieve 脚本。
admin-settings-key-cap-sieve-forward-external = 转发到账户域名之外
admin-settings-help-cap-sieve-forward-external = 允许脚本进行外部转发。保存这些脚本还需要近期验证。
admin-settings-key-session-max-concurrent = 并发会话
admin-settings-help-session-max-concurrent = 每个账户的最大会话数。新登录超出此限制时，最早的会话将被撤销。
admin-settings-key-session-idle-secs = 会话空闲超时
admin-settings-help-session-idle-secs = 空闲会话在这么多秒后结束，不超过部署上限。
admin-settings-key-session-absolute-secs = 会话有效期
admin-settings-help-session-absolute-secs = 会话在这么多秒后结束，不超过部署上限。
admin-settings-key-twofa-remember-days = 已记住设备的有效期
admin-settings-help-twofa-remember-days = 已记住的设备需要再次验证第二因素之前的天数。0 表示停用此功能。
admin-settings-key-twofa-passkeys-enabled = 允许通行密钥
admin-settings-help-twofa-passkeys-enabled = 允许将通行密钥用作身份验证因素。
admin-settings-key-twofa-required = 要求双因素身份验证
admin-settings-help-twofa-required = 在签发会话之前，要求已登记第二因素。
admin-settings-key-twofa-passkey-attachment = 通行密钥验证器类型
admin-settings-help-twofa-passkey-attachment = any（默认）：所有验证器。platform：存储在设备上的通行密钥（Windows Hello、Touch ID、Android 屏幕锁）。cross-platform：安全密钥和手机。此信息由浏览器报告，验证器不对其签名。
admin-settings-key-twofa-passkey-user-verification = 通行密钥用户验证
admin-settings-help-twofa-passkey-user-verification = required：通行密钥在添加时以及每次登录时都必须检查 PIN 或生物特征；没有此检查的流程会被拒绝。preferred（默认）和 discouraged 只改变向浏览器发出的请求。
admin-settings-key-twofa-passkey-resident-key = 可发现的通行密钥
admin-settings-help-twofa-passkey-resident-key = required：凭据必须存储在验证器上。discouraged（默认）或 preferred。在引擎模式下，通行密钥始终可发现，因为它们也可用于免密码登录。
admin-settings-key-twofa-passkey-legacy = 在当前策略之前添加的通行密钥
admin-settings-help-twofa-passkey-legacy = allow（默认）：不符合策略的旧通行密钥仍可使用，并提示用户更换。refuse：登录时拒绝并显示消息；用户改用其他因素或恢复码登录。
admin-settings-key-twofa-last-factor-removal = 移除最后一个第二因素
admin-settings-help-twofa-last-factor-removal = 当移除通行密钥或验证器应用会使账户没有任何第二因素时。allow（默认）：允许移除，之后登录只需密码。block：用户必须先添加另一种登录方式（恢复码不计）。在要求双因素身份验证的情况下，始终保留最后一个。
admin-settings-key-privacy-remote-content = 远程内容
admin-settings-help-privacy-remote-content = 阻止图片、先询问、使用代理，或在允许时直接加载。
admin-settings-key-privacy-trackers = 阻止追踪器
admin-settings-help-privacy-trackers = 阻止已知的邮件追踪图片。
admin-settings-key-privacy-clean-links = 清理链接
admin-settings-help-privacy-clean-links = 移除邮件链接中的追踪参数。
admin-settings-key-privacy-max-bytes = 远程图片大小限制
admin-settings-help-privacy-max-bytes = 获取单张远程图片的最大字节数。
admin-settings-key-privacy-cache-bytes = 远程图片缓存
admin-settings-help-privacy-cache-bytes = 远程图片缓存中保留的最大字节数。
admin-settings-key-privacy-rate-per-min = 远程图片请求速率
admin-settings-help-privacy-rate-per-min = 每个账户每分钟获取远程图片的最大次数。
admin-settings-privacy-fetch-title = 远程图片获取
admin-settings-privacy-fetch-help = 服务器为阅读者获取图片时的时间限制、重定向和并行下载。超出限制的图片会显示占位符。
admin-settings-key-privacy-fetch-connect-timeout-ms = 远程图片连接超时
admin-settings-help-privacy-fetch-connect-timeout-ms = 与图片主机建立连接（包括 TLS 握手）允许的毫秒数。默认 5000。
admin-settings-key-privacy-fetch-headers-timeout-ms = 远程图片响应超时
admin-settings-help-privacy-fetch-headers-timeout-ms = 从请求开始到图片主机返回响应头允许的毫秒数。默认 10000。
admin-settings-key-privacy-fetch-idle-timeout-ms = 远程图片停滞超时
admin-settings-help-privacy-fetch-idle-timeout-ms = 图片下载中两个数据块之间允许的最长停顿（毫秒），超过则停止。默认 10000。
admin-settings-key-privacy-fetch-total-timeout-ms = 远程图片总超时
admin-settings-help-privacy-fetch-total-timeout-ms = 获取一张图片（包括重定向）允许的毫秒数。排队等待有单独的预算，因此繁忙的图片不会在开始之前耗尽其网络时间。默认 20000。
admin-settings-key-privacy-fetch-queue-timeout-ms = 远程图片队列超时
admin-settings-help-privacy-fetch-queue-timeout-ms = 图片在其获取预算之外等待容量的最大毫秒数。有界队列可应对大量已认证的突发请求，同时仍会强制执行活动获取和解码限制。默认 60000。
admin-settings-key-privacy-max-redirects = 远程图片重定向
admin-settings-help-privacy-max-redirects = 单张图片可跟随的重定向次数。每次重定向都按与首次请求相同的地址规则检查。默认 3。
admin-settings-key-privacy-max-concurrent-per-message = 每封邮件的远程图片下载数
admin-settings-help-privacy-max-concurrent-per-message = 同一封邮件中同时获取的图片数。其余的等待空闲名额。范围 1–128；默认 30。
admin-settings-key-privacy-max-concurrent-per-account = 每个账户的远程图片下载数
admin-settings-help-privacy-max-concurrent-per-account = 一个账户在其所有已打开的邮件中同时获取的图片数。范围 1–256；默认 128。
admin-settings-key-privacy-allow-direct = 允许直接加载图片
admin-settings-key-cap-remote-content-direct = 直接加载图片的权限
admin-settings-help-cap-remote-content-direct = 在服务器允许时，允许此账户选择直接加载图片。
admin-settings-help-privacy-allow-direct = 允许具备所需权限的账户直接加载图片。
admin-settings-key-logs-auth-retention-days = 身份验证历史保留期
admin-settings-help-logs-auth-retention-days = 身份验证元数据保留的天数，与持久的账户身份无关。
admin-settings-key-logs-auth-max-entries = 身份验证历史条目上限
admin-settings-help-logs-auth-max-entries = 保留的最大条目数。一分钟内相同的重复失败共用一个条目。
admin-settings-key-reader-color-mode = 邮件颜色
admin-settings-cat-reader = 邮件显示
admin-settings-key-mail-row-actions-mode = 邮件列表快捷操作
admin-settings-help-mail-row-actions-mode = 每个邮件行上快捷操作的显示方式：open（始终可用，悬停或聚焦时显示）、hover（三个控件和一个“更多”按钮）、click（一个用来展开它们的按钮）或 off（从邮件行中移除；上下文菜单和工具栏仍保留所有操作）。在触摸屏上，hover 的行为与 click 相同。除非管理员将其锁定，用户可以自行选择。
admin-settings-key-mail-quick-reply = 快速回复
admin-settings-help-mail-quick-reply = 提供一个已填好收件人的精简回复标签页。默认关闭；除非被管理员锁定，用户可以启用它。正常的发送保护仍然有效。
admin-settings-key-mail-request-read-receipt = 默认请求已读回执
admin-settings-help-mail-request-read-receipt = 对受支持的外发邮件，默认选中回执请求。这只是撰写时的默认值，并不保证收件人会发送回执。
admin-settings-help-reader-color-mode = 自动会跟随深色主题。可在隔离的邮件框架内选择原始、自适应或反转的渲染方式。
admin-settings-key-reader-font-size = 邮件字体大小
admin-settings-help-reader-font-size = 默认阅读字号，单位为像素，范围 12 到 24。
admin-settings-key-reader-line-height = 邮件行高
admin-settings-help-reader-line-height = 阅读行距，范围 1.2 到 2.2。
admin-settings-key-reader-content-width = 邮件阅读宽度
admin-settings-help-reader-content-width = 邮件内容为舒适、宽或整个窗格宽度。
admin-settings-key-reader-loading = 邮件加载外观
admin-settings-help-reader-loading = 占位行、文字指示或无视觉指示。无障碍播报仍然可用。
admin-settings-key-reader-loading-delay-ms = 邮件加载延迟
admin-settings-help-reader-loading-delay-ms = 仅延迟加载指示，范围 0 到 1000 毫秒。邮件的送达从不被延迟。
admin-settings-key-reader-zoom = 邮件缩放
admin-settings-help-reader-zoom = 默认邮件正文缩放百分比，范围 50 到 200；阅读器使用最接近的缩放级别。只缩放邮件，打印时使用 100%。
admin-settings-key-reader-attachments-collapsed = 附件栏折叠
admin-settings-help-reader-attachments-collapsed = 阅读器的附件栏是否初始折叠，只显示数量和总大小。用户最近一次的展开或折叠会保存在此处。默认关闭：显示附件。
admin-settings-key-login-show-domain-hint = 在登录页显示首选域名
admin-settings-help-login-show-domain-hint = 显示哪个域名接受不带邮箱后缀的用户名。默认隐藏。这只改变提示；使用用户名登录仍然有效。
admin-settings-key-signin-retry-cooldown-secs = 登录重试冷却时间
admin-settings-help-signin-retry-cooldown-secs = 同一客户端网络两次密码登录尝试之间的最少秒数。默认 3；0 表示停用冷却。现有的失败封禁仍然适用。
admin-settings-key-mail-quote-style = 回复和转发的邮件头样式
admin-settings-key-mail-external-recipient-warning = 外部收件人警告
admin-settings-help-mail-external-recipient-warning = 关闭、警告，或在撰写窗口中要求确认。涵盖收件人、抄送和密送。这是撰写窗口的保护措施，不是服务器的投递限制。
admin-settings-key-mail-trusted-recipient-domains = 受信任的收件人域名
admin-settings-help-mail-trusted-recipient-domains = 最多 100 个完全匹配的收件人域名。所选发件身份的域名也受信任。无效条目不会获得信任。锁定此设置可防止用户覆盖。
admin-settings-help-mail-quote-style = 使用经典的引用式回复，或 Outlook 风格的邮件头。这设定用户的默认值；锁定可强制使用该样式。现有草稿不变。
admin-settings-key-signin-anti-bot-mode = 登录机器人防护
admin-settings-help-signin-anti-bot-mode = 自托管的工作量证明验证：off、mailbox、admin 或 all。不会联系任何外部验证服务。
admin-settings-key-signin-anti-bot-difficulty = 登录验证工作量
admin-settings-help-signin-anti-bot-difficulty = 求解器计数器的最大值，100–20000。值越高，浏览器需要完成的工作越多；默认 2000。
admin-settings-key-signin-anti-bot-expiry-secs = 登录验证有效期
admin-settings-help-signin-anti-bot-expiry-secs = 一次性验证保持有效的秒数，60–600；默认 180。每次尝试凭据之后都需要新的验证。
admin-settings-key-attachments-preview-allowed = 允许附件预览
admin-settings-help-attachments-preview-allowed = 关闭可阻止附件预览渲染；原始下载有自己的控制项。
admin-settings-key-attachments-download-allowed = 允许提供原始附件
admin-settings-help-attachments-download-allowed = 关闭后也会阻止需要原始字节的预览。浏览器预览不是防复制保护。
admin-settings-key-attachments-max-bytes = 附件最大字节数
admin-settings-help-attachments-max-bytes = 0 表示不限制。适用于提供的原始字节，包括范围请求。
admin-settings-key-attachments-allowed-extensions = 允许的文件扩展名
admin-settings-help-attachments-allowed-extensions = 小写扩展名（不带点）的 JSON 数组，例如 ["pdf","tar.gz"]。留空表示允许任何扩展名；阻止规则优先。
admin-settings-key-attachments-blocked-extensions = 阻止的文件扩展名
admin-settings-help-attachments-blocked-extensions = 小写扩展名的 JSON 数组。按已存储的 MIME 文件名判断，从不使用下载 URL 中的名称。
admin-settings-key-attachments-warn-extensions = 需警告的文件扩展名
admin-settings-help-attachments-warn-extensions = 在提供原始文件之前需要明确确认的扩展名 JSON 数组。
admin-settings-key-attachments-allowed-mime-types = 允许的 MIME 类型
admin-settings-help-attachments-allowed-mime-types = 小写类型或顶级通配符的 JSON 数组，例如 ["image/*","application/pdf"]。留空表示允许任何类型。
admin-settings-key-attachments-blocked-mime-types = 阻止的 MIME 类型
admin-settings-help-attachments-blocked-mime-types = 小写类型或顶级通配符的 JSON 数组。阻止规则优先。
admin-settings-key-attachments-warn-mime-types = 需警告的 MIME 类型
admin-settings-help-attachments-warn-mime-types = 需要确认的小写类型或顶级通配符的 JSON 数组。MIME 元数据不是恶意软件检测。
admin-settings-cat-shortcuts = 键盘快捷键
admin-settings-cat-time = 时区
admin-settings-key-time-display-zone = 显示时区
admin-settings-help-time-display-zone = IANA 时区，如 Europe/Lisbon，或填 local 以跟随当前设备。邮件时间戳仍按其原始时刻存储。
admin-settings-key-time-comparison-zone = 对比时区
admin-settings-help-time-comparison-zone = 供邮件和日历辅助功能使用的可选第二个 IANA 时区。留空则隐藏对比；不会根据邮箱地址推断位置。
admin-settings-key-time-show-helpers = 显示时区辅助信息
admin-settings-help-time-show-helpers = 为邮件时间戳和日历事件显示本地时间与所选时区的对比。管理员锁定可强制此选择。
admin-settings-key-shortcuts-enabled = 启用键盘快捷键
admin-settings-help-shortcuts-enabled = 允许应用快捷键。锁定可在全局或针对某个域名停用它们。输入框和模态对话框保持其正常的键盘处理。
admin-settings-key-shortcuts-compose = 撰写快捷键
admin-settings-help-shortcuts-compose = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-archive = 归档快捷键
admin-settings-help-shortcuts-archive = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-reply = 回复快捷键
admin-settings-help-shortcuts-reply = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-reply-all = 全部回复快捷键
admin-settings-help-shortcuts-reply-all = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-forward = 转发快捷键
admin-settings-help-shortcuts-forward = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-next = 下一封邮件快捷键
admin-settings-help-shortcuts-next = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-previous = 上一封邮件快捷键
admin-settings-help-shortcuts-previous = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-search = 聚焦搜索快捷键
admin-settings-help-shortcuts-search = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-refresh = 刷新邮件快捷键
admin-settings-help-shortcuts-refresh = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-settings = 打开设置快捷键
admin-settings-help-shortcuts-settings = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-trash = 移到废纸篓快捷键
admin-settings-help-shortcuts-trash = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-mark-read = 标为已读快捷键

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-shortcuts-mark-read = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-mark-unread = 标为未读快捷键
admin-settings-help-shortcuts-mark-unread = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-spam = 标为垃圾邮件快捷键
admin-settings-help-shortcuts-spam = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-zoom-in = 放大邮件快捷键
admin-settings-help-shortcuts-zoom-in = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-zoom-out = 缩小邮件快捷键
admin-settings-help-shortcuts-zoom-out = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-zoom-reset = 重置邮件缩放快捷键
admin-settings-help-shortcuts-zoom-reset = 使用 inherit 表示沿用所选预设，disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-shortcuts-open-in-tab = 在独立标签页中打开邮件快捷键
admin-settings-help-shortcuts-open-in-tab = 使用 inherit 表示沿用所选预设（Shift+Enter），disabled 表示移除此操作，或使用组合键，如 Ctrl+Shift+K。冲突或无效的组合键不会生效。
admin-settings-key-notifications-push-allowed = 允许后台推送通知
admin-settings-help-notifications-push-allowed = 允许此账户已启用的通知注册并接收后台推送。仍需要浏览器权限。
admin-settings-key-offline-persistent-storage-allowed = 允许请求浏览器持久存储
admin-settings-help-offline-persistent-storage-allowed = 允许发出明确的请求，以保护本地缓存数据不被浏览器自动清除。浏览器可能拒绝；这不会自动授予权限。
admin-settings-key-security-signin-mailbox-failure-limit = 邮箱凭据失败次数上限
admin-settings-help-security-signin-mailbox-failure-limit = 在统计窗口内，来自同一源 IP 的凭据被拒次数达到上限，会触发邮箱登录的临时锁定。网络和配置故障不计入。
admin-settings-key-security-signin-admin-failure-limit = 管理员凭据失败次数上限
admin-settings-help-security-signin-admin-failure-limit = 来自同一源 IP 的管理员凭据被拒，会触发单独的管理员登录临时锁定。较短的管理员会话和新近证明的要求保持不变。
admin-settings-key-security-signin-failure-window-secs = 凭据失败统计窗口
admin-settings-help-security-signin-failure-window-secs = 凭据和多因素验证被拒的滚动窗口，单位为秒。成功完成登录会重置该来源和该入口的计数。
admin-settings-key-security-signin-lock-secs = 临时登录锁定时长
admin-settings-help-security-signin-lock-secs = 锁定时长，单位为秒。锁定期间的重复尝试不会延长其原有的到期时间。手动设置的全局 IP 封禁与此无关。
admin-settings-key-security-signin-geo-scope = 登录国家/地区策略范围
admin-settings-help-security-signin-geo-scope = 默认关闭。仅适用于新的邮箱登录、管理员登录或所有登录。现有会话不会被撤销。启用前请检查当前来源诊断；本地 GeoIP 数据库不可用时将拒绝登录。
admin-settings-key-security-signin-geo-mode = 国家/地区允许或拒绝列表
admin-settings-help-security-signin-geo-mode = allow 只允许列出的国家/地区；deny 拒绝列出的国家/地区。未知地址使用其自己的明确策略。
admin-settings-key-security-signin-geo-countries = 登录国家/地区代码
admin-settings-help-security-signin-geo-countries = 大写两位国家/地区代码的 JSON 数组，例如 ["PT", "GB"]。仅使用本地国家/地区数据库（安全 > 国家/地区数据库，或 MW_GEOIP_DB）；地址绝不会发送到位置服务。
admin-settings-key-security-signin-geo-unknown = 未映射来源地址策略
admin-settings-help-security-signin-geo-unknown = 允许或拒绝可读本地数据库中不存在的地址，包括私有地址。即使允许未知地址，数据库缺失或不可读时，仍总会拒绝适用策略的登录。
admin-settings-key-contacts-avatar-mode = 联系人头像与自动生成的头像
admin-settings-help-contacts-avatar-mode = 在本地生成首字母头像、隐藏头像，或使用各账户自己通讯录中的内嵌照片。不会使用远程照片 URL，也不会联系外部头像服务。用户和域名策略可由管理员锁定。
admin-settings-cat-contacts = 联系人头像
admin-settings-key-reader-action-style = 邮件操作功能区
admin-settings-help-reader-action-style = 默认使用紧凑图标。选择“标签”可显示带文字的图标；两者都保留操作名称和主题化的工具提示。支持全局、域名和用户策略锁定。
admin-settings-key-nextcloud-sync-interval-secs = Nextcloud 自动刷新间隔
admin-settings-help-nextcloud-sync-interval-secs = 已连接的原生引擎账户在后台刷新集合的间隔秒数。0 表示仅手动；正值最小为六十秒。失败后会退避。不会尝试后台登录。
admin-settings-key-mail-folder-role-inbox = 收件箱文件夹映射
admin-settings-help-mail-folder-role-inbox = Null 使用自动发现。共享默认值使用精确路径对象，单个账户使用其拥有的邮箱 ID 对象。缺失或冲突的目标会停用此角色；管理员锁定仍然适用。
admin-settings-key-mail-folder-role-drafts = 草稿文件夹映射
admin-settings-help-mail-folder-role-drafts = Null 使用自动发现。共享默认值使用精确路径对象，单个账户使用其拥有的邮箱 ID 对象。缺失或冲突的目标会停用此角色；管理员锁定仍然适用。
admin-settings-key-mail-folder-role-sent = 已发送文件夹映射
admin-settings-help-mail-folder-role-sent = Null 使用自动发现。共享默认值使用精确路径对象，单个账户使用其拥有的邮箱 ID 对象。缺失或冲突的目标会停用此角色；管理员锁定仍然适用。
admin-settings-key-mail-folder-role-trash = 废纸篓文件夹映射
admin-settings-help-mail-folder-role-trash = Null 使用自动发现。共享默认值使用精确路径对象，单个账户使用其拥有的邮箱 ID 对象。缺失或冲突的目标会停用此角色；管理员锁定仍然适用。
admin-settings-key-mail-folder-role-junk = 垃圾邮件文件夹映射
admin-settings-help-mail-folder-role-junk = Null 使用自动发现。共享默认值使用精确路径对象，单个账户使用其拥有的邮箱 ID 对象。缺失或冲突的目标会停用此角色；管理员锁定仍然适用。
admin-settings-key-mail-folder-role-archive = 归档文件夹映射
admin-settings-help-mail-folder-role-archive = Null 使用自动发现。共享默认值使用精确路径对象，单个账户使用其拥有的邮箱 ID 对象。缺失或冲突的目标会停用此角色；管理员锁定仍然适用。
admin-settings-key-mail-request-delivery-receipt = 默认请求投递通知
admin-settings-help-mail-request-delivery-receipt = 当账户支持 SMTP DSN 时，为新邮件默认选中投递通知。锁定控制的是这个已保存的默认值；receipts.dsnAllowed 另行决定是否允许发出请求。投递并不证明已被阅读。
admin-settings-key-security-signin-geo-enforce-sessions = 对现有会话强制执行国家/地区策略
admin-settings-help-security-signin-geo-enforce-sessions = 默认关闭。将已配置的登录国家/地区范围应用于已认证请求、原生 bearer 访问和已签名的阅读内容。被阻止的客户端仍可退出登录。已打开的流在重新连接时检查；这不会终止现有连接。
admin-settings-key-security-protection-rate-mode = 汇总请求预算
admin-settings-help-security-protection-rate-mode = 默认关闭。observe 只记录预算耗尽而不拒绝；enforce 返回临时 429。现有的协议并发限制保持独立。
admin-settings-key-security-protection-adaptive-mode = 自适应请求防护
admin-settings-help-security-protection-adaptive-mode = 默认关闭。observe 记录可解释的类型化异常和建议的措施；enforce 应用有时限的临时限流或封禁。这是确定性的计数器策略，不是 AI 分类器。
admin-settings-key-security-protection-client-requests = 每个窗口的客户端请求数
admin-settings-help-security-protection-client-requests = 来自同一未登录地址的每个窗口请求数，以及每个登录请求。持有有效邮箱或管理员会话的请求使用其自己更大的额度。0 表示关闭此预算。
admin-settings-key-security-protection-account-requests = 每个窗口的账户请求数
admin-settings-help-security-protection-account-requests = 一个已登录邮箱在所有会话、设备和地址上的每个窗口请求数。设置得高于匿名额度，让已登录用户有更多余地。0 表示关闭此预算。
admin-settings-key-rate-limit-diagnostics-burst = 同时运行的诊断数
admin-settings-help-rate-limit-diagnostics-burst = 管理员会话可连续启动的诊断运行和后端检查数。启动一个已在运行的检查会返回该检查，而不会重复计数。
admin-settings-key-rate-limit-diagnostics-per-minute = 每分钟诊断运行数
admin-settings-help-rate-limit-diagnostics-per-minute = 管理员会话每分钟恢复的诊断运行和后端检查数。
admin-settings-key-security-protection-admin-requests = 每个窗口的管理员请求数
admin-settings-help-security-protection-admin-requests = 一位已登录管理员的每个窗口请求数。设置得高于匿名额度。0 表示关闭此预算。
admin-settings-key-security-protection-rate-window-secs = 请求预算窗口（秒）
admin-settings-help-security-protection-rate-window-secs = 固定的请求预算间隔。容量卸载和普通的未找到响应不属于自适应依据。
admin-settings-key-security-protection-adaptive-clients = 为自适应防护观察客户端
admin-settings-help-security-protection-adaptive-clients = 在自适应模式启用时启用此对象维度。账户和管理员仅在其身份验证通过后才计数。
admin-settings-key-security-protection-adaptive-accounts = 为自适应防护观察账户
admin-settings-help-security-protection-adaptive-accounts = 在自适应模式启用时启用此对象维度。账户和管理员仅在其身份验证通过后才计数。
admin-settings-key-security-protection-adaptive-admins = 为自适应防护观察管理员
admin-settings-help-security-protection-adaptive-admins = 在自适应模式启用时启用此对象维度。账户和管理员仅在其身份验证通过后才计数。
admin-settings-key-security-protection-short-window-secs = 短期异常窗口（秒）
admin-settings-help-security-protection-short-window-secs = 滚动观察窗口。长窗口支持数小时至三十天；分桶取整最多可多保留一个窗口的六十四分之一。
admin-settings-key-security-protection-short-minimum-samples = 短期比例最少样本数
admin-settings-help-security-protection-short-minimum-samples = 比例触发条件生效前，所观察到的正常请求加异常请求的最少数量。绝对异常阈值独立于此最小值。
admin-settings-key-security-protection-short-absolute-anomalies = 短期绝对异常预算
admin-settings-help-security-protection-short-absolute-anomalies = 当累积了这么多所选异常时触发，即使正常请求稀释了比例也是如此。0 表示停用此触发条件。
admin-settings-key-security-protection-short-ratio-percent = 短期异常比例（百分比）
admin-settings-help-security-protection-short-ratio-percent = 在达到最少样本数之后，当所选异常达到此百分比时触发。绝对条件或比例条件任一满足即可。0 表示停用此触发条件。
admin-settings-key-security-protection-long-window-secs = 长期异常窗口（秒）
admin-settings-help-security-protection-long-window-secs = 滚动观察窗口。长窗口支持数小时至三十天；分桶取整最多可多保留一个窗口的六十四分之一。
admin-settings-key-security-protection-long-minimum-samples = 长期比例最少样本数
admin-settings-help-security-protection-long-minimum-samples = 比例触发条件生效前，所观察到的正常请求加异常请求的最少数量。绝对异常阈值独立于此最小值。
admin-settings-key-security-protection-long-absolute-anomalies = 长期绝对异常预算
admin-settings-help-security-protection-long-absolute-anomalies = 当累积了这么多所选异常时触发，即使正常请求稀释了比例也是如此。0 表示停用此触发条件。
admin-settings-key-security-protection-long-ratio-percent = 长期异常比例（百分比）
admin-settings-help-security-protection-long-ratio-percent = 在达到最少样本数之后，当所选异常达到此百分比时触发。绝对条件或比例条件任一满足即可。0 表示停用此触发条件。
admin-settings-key-security-protection-normal-cap-per-bucket = 每个分桶的正常基线上限
admin-settings-help-security-protection-normal-cap-per-bucket = 限制每个滚动窗口分桶中学到的正常基线。独立的绝对异常预算可防止无限量的有效请求填充来掩盖滥用。
admin-settings-key-security-protection-signals = 自适应依据信号
admin-settings-help-security-protection-signals = 选择权威的格式错误、体积过大、凭据被拒、权限被拒、请求体超时，或需主动启用的管理路径未命中观察。管理路径探测不包括实体未找到的响应和静态资源；通用状态码、容量拒绝和 User-Agent 字符串不能证明存在滥用。
admin-settings-key-security-protection-throttle-secs = 初始临时限流（秒）
admin-settings-help-security-protection-throttle-secs = 首次越过阈值时开始一个固定的限流截止时间。截止时间内的请求不会将其延长。
admin-settings-key-security-protection-block-secs = 升级后的临时封禁（秒）
admin-settings-help-security-protection-block-secs = 在恢复之前再次越过阈值，可能升级为此有时限的封禁。不会创建永久账户锁定。
admin-settings-key-security-protection-decay-secs = 静默恢复间隔（秒）
admin-settings-help-security-protection-decay-secs = 经过这段无异常的间隔并且当前措施到期后，升级会回到初始阶段。滚动依据按其配置的窗口过期。
admin-settings-key-security-protection-exempt-clients = 豁免的客户端地址和网络
admin-settings-help-security-protection-exempt-clients = 精确的 IP 地址或 CIDR 网络，仅豁免客户端维度。已验证账户的限制仍然适用。共享 NAT 网络请谨慎使用；转发的地址仅在经由已配置代理时才被信任。
admin-settings-key-security-protection-exempt-accounts = 豁免的已验证账户 ID
admin-settings-help-security-protection-exempt-accounts = 服务器已验证的邮箱账户 ID，仅豁免其账户维度，绝不豁免来源客户端维度。邮箱地址和调用方提供的账户参数不属于身份证据。
admin-settings-key-security-protection-exempt-admins = 豁免的管理员 ID
admin-settings-help-security-protection-exempt-admins = 已通过认证的管理员 ID，仅豁免其管理员维度。现有的登录失败锁定和严格的管理员会话期限保持不变。
admin-settings-key-security-protection-storage-failure = 强制执行存储故障策略
admin-settings-help-security-protection-storage-failure = closed 在已启用的强制预算无法检查时拒绝受保护的工作。open 在发出有限的运行警告的同时临时放行。仅观察模式的故障绝不会施加强制措施。
admin-settings-key-security-admin-probe-enabled = 管理路径探测防护
admin-settings-help-security-admin-probe-enabled = 按客户端网络（IPv4 地址、IPv6 /64）统计查找管理控制台的请求，对持续探测的来源减速，然后封禁。拼写错误和过期书签保持在宽限额度之内。在安全 > 自适应防护中更改。
admin-settings-key-security-admin-probe-grace-allowance = 宽限额度（每个窗口的未命中数）
admin-settings-help-security-admin-probe-grace-allowance = 每个窗口内忽略的未命中数。得分以每个窗口这么多分的速率持续衰减。
admin-settings-key-security-admin-probe-window-secs = 宽限窗口（秒）
admin-settings-help-security-admin-probe-window-secs = 宽限额度所适用的窗口。
admin-settings-key-security-admin-probe-block-after = 封禁前超出宽限的未命中数
admin-settings-help-security-admin-probe-block-after = 超过宽限额度后触发封禁的未命中数。介于两者之间的未命中会在拖延延迟后才得到响应。
admin-settings-key-security-admin-probe-tarpit-delay-ms = 拖延延迟（毫秒）
admin-settings-help-security-admin-probe-tarpit-delay-ms = 超出宽限额度的每次计数未命中所增加的延迟。0 表示关闭拖延。其他请求绝不会被延迟。
admin-settings-key-security-admin-probe-block-secs = 首次封禁（秒）
admin-settings-help-security-admin-probe-block-secs = 首次封禁的时长。被封禁的来源对每个路径得到相同的响应。
admin-settings-key-security-admin-probe-max-block-secs = 最长封禁（秒）
admin-settings-help-security-admin-probe-max-block-secs = 重复封禁会将上一次的时长加倍，直至此上限。
admin-settings-key-security-admin-probe-repeat-window-secs = 重复窗口（秒）
admin-settings-help-security-admin-probe-repeat-window-secs = 在上一次封禁结束后的这段时间内开始的封禁，计为重复封禁。
admin-settings-key-security-admin-probe-block-status = 封禁响应状态
admin-settings-help-security-admin-probe-block-status = 发送给被封禁来源的 HTTP 状态，并附带 Retry-After：429 或 403。
admin-settings-key-security-admin-probe-signals = 计入的未命中
admin-settings-help-security-admin-probe-signals = probe-path、slug-guess（在控制台路径深度处的 404）、admin-signin-failed 和 admin-unauthenticated。
admin-settings-key-security-admin-probe-probe-paths = 探测路径
admin-settings-help-security-admin-probe-probe-paths = 扫描器会请求的路径，如 /wp-admin 或 /.env。包含其中某个完整路径段的路径计为一次未命中。控制台自身的路径绝不计入。
admin-settings-key-security-admin-probe-allowlist = 永不封禁的网络
admin-settings-help-security-admin-probe-allowlist = 永不计数也永不封禁的 IP 地址或 CIDR 网络（办公室、VPN）。
admin-settings-key-security-admin-probe-audit-ip = 审计日志中的来源地址
admin-settings-help-security-admin-probe-audit-ip = masked 显示 IPv4 /24 和 IPv6 /48；full 显示完整地址。fail2ban 日志行始终包含完整地址。
admin-settings-key-security-admin-probe-fail2ban-log = 封禁时写入 fail2ban 日志行
admin-settings-help-security-admin-probe-fail2ban-log = 来源被封禁时，写入一行带有 rhost=<address> 的 mailwoman[admin-probe]。
admin-settings-key-appearance-custom-css-enabled = 允许个人外观 CSS
admin-settings-help-appearance-custom-css-enabled = 允许在所有者的工作区中使用受限的视觉样式。默认关闭；强制执行的邮箱外观锁定会暂停这些样式。
admin-settings-key-appearance-custom-js-enabled = 允许个人外观脚本
admin-settings-help-appearance-custom-js-enabled = 允许只能请求外观更改的隔离脚本。默认关闭；脚本无法访问邮件、账户数据或应用的 DOM。
admin-settings-key-mail-modifier-selection = 邮件修饰键选择
admin-settings-help-mail-modifier-selection = 允许使用 Ctrl、Command 和 Shift 进行范围选择。锁定此值可强制执行；可见的选择控件仍然可用。
admin-settings-key-mail-drag-to-folder = 将邮件拖到文件夹
admin-settings-help-mail-drag-to-folder = 允许在账户范围内将邮件拖到真实文件夹。这不会授予邮箱写入权限；服务器仍会对每次移动进行授权。
admin-settings-key-admin-allow-last-superadmin-loss = 允许失去最后一个超级管理员
admin-settings-help-admin-allow-last-superadmin-loss = 默认关闭。启用后，之后的更改可以移除最后的超级管理员访问权限。必须由完整的超级管理员通过新的身份验证确认后果。
admin-settings-last-superadmin-title = 允许失去所有超级管理员权限？
admin-settings-last-superadmin-body = 之后的账户更改可能导致没有人能够管理此部署。恢复需要进行本地部署管理。输入 ALLOW LAST SUPERADMIN LOSS 以启用此例外。
admin-settings-last-superadmin-confirm = 允许失去访问权限
admin-settings-key-mail-folder-counts = 文件夹计数
admin-settings-help-mail-folder-counts = 选择在文件夹旁边隐藏计数、显示未读数、仅在有未读邮件时显示未读数、显示总数，或两者都显示。可设置全局、域名或用户默认值，并在需要时锁定该选择。适用于账户的每个关联邮箱。
admin-settings-key-mail-folder-count-scope = 显示计数的文件夹
admin-settings-help-mail-folder-count-scope = 所有文件夹，或仅主要文件夹（收件箱、草稿、已发送、归档、垃圾邮件、废纸篓、发件箱及其他特殊用途文件夹）。仅选主要文件夹时，自定义文件夹及其子文件夹不显示计数。适用于账户的每个关联邮箱。
admin-settings-key-mail-compact-folder-badges = 紧凑的次要文件夹徽章
admin-settings-help-mail-compact-folder-badges = 在收件箱之外显示较小的徽章。除非此设置被锁定，用户可以选择自己的大小。
admin-settings-key-mail-focused-inbox = 重点收件箱
admin-settings-help-mail-focused-inbox = 将收件箱拆分为“重点”和“其他”标签页；新闻邮件和自动发件人会归入“其他”。默认关闭。除非此设置被锁定，用户可以在设置 > 文件夹和侧边栏中开启或关闭。
admin-settings-key-mail-account-unread-badge = 邮箱名称旁的未读数
admin-settings-help-mail-account-unread-badge = 在侧边栏中已登录地址或昵称旁显示收件箱的未读数。默认关闭。邮箱菜单中其他邮箱的数量始终显示。适用于账户的每个关联邮箱。
admin-settings-key-privacy-remote-image-downloads = 手动下载远程图片
admin-settings-key-privacy-remote-images-toast = 图片加载通知
admin-settings-help-privacy-remote-images-toast = 当邮件的远程图片通过隐私代理加载时，在阅读器中显示一条通知，并在加载完成时或有多少张无法加载时给出提示。默认开启。这只改变阅读器的显示；加载哪些图片仍由远程内容策略决定。
admin-settings-help-privacy-remote-image-downloads = 允许通过受保护的隐私代理，明确下载所属邮件引用的图片。默认关闭；这绝不会直接打开外部 URL。
admin-settings-key-mail-resume-tabs = 恢复工作区标签页
admin-settings-help-mail-resume-tabs = 重新加载后恢复账户所属的标签页引用。停用后会清除本地工作区记录，但不会删除草稿。
admin-settings-key-metrics-default-range-seconds = 默认请求历史范围
admin-settings-help-metrics-default-range-seconds = 历史请求未指定范围时显示的秒数。受保留期和最大范围限制。
admin-settings-key-metrics-max-range-seconds = 最大请求历史范围
admin-settings-help-metrics-max-range-seconds = 每次历史查询的最大秒数。保留期仍是额外的限制。
admin-settings-key-contacts-favorites-first = 建议中收藏联系人优先
admin-settings-help-contacts-favorites-first = 在收件人建议中，将收藏的联系人排在其他匹配项之前。只改变顺序；不会隐藏任何建议。用户和域名覆盖可以被锁定。
admin-settings-key-contacts-sender-icons = 内置发件人图标
admin-settings-help-contacts-sender-icons = 为已识别的发件人域名显示本地内置图标，其他则显示自动生成的首字母。联系人头像优先。图标不会验证发件人的身份。不会联系外部徽标服务；用户和域名覆盖可以被锁定。
admin-settings-key-mail-brand-avatars = 已验证发件人旁的品牌徽标
admin-settings-help-mail-brand-avatars = 仅当邮件通过了针对该品牌自身域名的 DKIM 和对齐的 DMARC 检查时，才在阅读器中的发件人旁显示本地内置的品牌徽标。未通过或缺少这些检查的邮件保留普通头像。不会从发件人或徽标服务获取任何徽标。
admin-settings-key-mail-draft-save-mode = 账户草稿保存模式
admin-settings-help-mail-draft-save-mode = 在输入停顿后保存、定期保存，或仅按需保存。保存需要受支持的上游草稿文件夹；不确定的写入绝不会自动重试。
admin-settings-key-mail-draft-debounce-ms = 草稿输入停顿（毫秒）
admin-settings-help-mail-draft-debounce-ms = 在实时模式下，编辑后等待 500 到 10000 毫秒再保存。
admin-settings-key-mail-draft-interval-secs = 草稿保存间隔（秒）
admin-settings-help-mail-draft-interval-secs = 在定期模式下，每 5 到 3600 秒保存已编辑的草稿。未更改的草稿不会再次写入。
admin-settings-key-login-remember-signins = 记住成功登录的地址
admin-settings-help-login-remember-signins = 在此设备上最多保留十个成功使用过的用户名或邮箱地址，供登录下拉列表使用。绝不保存密码或会话。停用后，当设备下次加载登录页面或完成登录时，会移除此服务器保存的地址；这不控制浏览器自身的自动填充。
admin-settings-key-login-show-remembered-hint = 显示已保存地址的说明
admin-settings-help-login-show-remembered-hint = 在“以前的登录”下拉列表中显示说明文字。默认隐藏。仅在启用了记住成功登录时适用。
admin-settings-key-mail-show-from = 显示撰写窗口的发件人字段
admin-settings-help-mail-show-from = 在新的撰写窗口中默认显示发件人。用户可以为某封邮件显示它；只有获得授权的地址才能发送。
admin-settings-key-accounts-linking-enabled = 允许关联邮箱账户
admin-settings-help-accounts-linking-enabled = 默认停用。全局停用会阻止所有关联访问；两个账户还必须满足各自的域名和账户策略。关联需要对子账户进行实际身份验证，并提供新近的主账户证明。
admin-settings-key-accounts-use-linked = 使用关联邮箱
admin-settings-help-accounts-use-linked = 用户对关联访问的退出选项。无法覆盖管理员的停用。访问被停用时，邮箱数据会保留。
admin-settings-key-accounts-max-linked = 关联邮箱数量上限
admin-settings-help-accounts-max-linked = 每个主身份下已验证子邮箱的最大数量，范围 1 到 16。默认 5。
admin-settings-key-accounts-store-linked-credentials = 保持关联邮箱的访问
admin-settings-help-accounts-store-linked-credentials = 在服务器上密封保存每个关联邮箱的密码，使其访问能够跨会话和重启自动续期。零访问账户或启用了两步验证的邮箱绝不保存。关闭会清除已存储的密码；之后链接需重新登录才能续期。默认开启。
admin-settings-key-privacy-strip-mailer-headers = 移除外发邮件的邮件程序头
admin-settings-help-privacy-strip-mailer-headers = 从原生 SMTP 和 EWS 邮件中移除顶层的 X-Mailer、User-Agent 和 X-MimeOLE。附带的邮件和报告负载保持不变。不涵盖通过上游 JMAP 代理发送的邮件。
admin-settings-key-images-trusted-hosts = 受信任的图片主机
admin-settings-help-images-trusted-hosts = 最多 100 个图片主机及其子域名，当远程内容设为“询问”时，其远程图片无需逐封邮件授权即可加载。绝不会覆盖阻止模式；被阻止的主机仍被阻止。
admin-settings-key-images-blocked-hosts = 被阻止的图片主机
admin-settings-help-images-blocked-hosts = 最多 100 个图片主机及其子域名，其远程图片在任何模式下都不会加载。优先于受信任的主机和任何授权。
admin-settings-key-links-trusted-domains = 受信任的链接域名
admin-settings-help-links-trusted-domains = 最多 100 个精确域名，其链接从“检查链接”打开时无需目标确认步骤。不支持通配符、协议、端口或 IP 地址。纯 HTTP 链接始终需要检查。锁定此设置可防止用户更改列表。
admin-settings-key-links-trusted-subdomains = 信任受信任链接域名的子域名
admin-settings-help-links-trusted-subdomains = 关闭：仅信任精确的域名。开启：以 .<domain> 结尾的名称也受信任。按完整标签匹配，因此 example.com 绝不会信任 evilexample.com。
admin-settings-key-cap-compose-rich-text = 富文本撰写
admin-settings-help-cap-compose-rich-text = 用户是否可以撰写带格式（HTML）的邮件。关闭则只发送纯文本。
admin-settings-key-cap-compose-attachments = 新邮件中的附件
admin-settings-help-cap-compose-attachments = 用户是否可以为自己撰写的邮件添加附件。正文中的内嵌图片不计入。
admin-settings-key-cap-compose-share-links = 大文件共享链接
admin-settings-help-cap-compose-share-links = 用户是否可以将大文件以 Nextcloud 共享链接而不是附件的形式发送。
admin-settings-key-cap-compose-scheduled-send = 定时发送
admin-settings-help-cap-compose-scheduled-send = 用户是否可以安排邮件稍后发送。
admin-settings-key-cap-compose-read-receipts = 请求已读回执
admin-settings-help-cap-compose-read-receipts = 用户是否可以向收件人请求已读回执。
admin-settings-key-cap-signatures = 签名
admin-settings-help-cap-signatures = 用户是否可以创建和编辑自己的签名。
admin-settings-key-cap-identities = 代发身份
admin-settings-help-cap-identities = 用户是否可以添加和编辑自己用来发信的身份。
admin-settings-key-cap-keys = OpenPGP 和 S/MIME 密钥
admin-settings-help-cap-keys = 用户是否拥有“密钥”模块，并可以对邮件进行签名和加密。
admin-settings-key-cap-links-trusted = 编辑受信任的链接域名
admin-settings-help-cap-links-trusted = 用户是否可以更改自己的受信任链接域名列表。管理员列表仍然适用。
admin-settings-key-cap-zoom = 阅读器缩放
admin-settings-help-cap-zoom = 用户是否可以缩放邮件阅读器。
admin-settings-key-cap-mail-focused-inbox = 重点收件箱
admin-settings-help-cap-mail-focused-inbox = 用户是否可以开启“重点”和“其他”收件箱标签页。
admin-settings-key-cap-views-events = 事件视图
admin-settings-help-cap-views-events = 用户是否拥有日历邮件的“事件”视图。
admin-settings-key-cap-views-attachments = 附件视图
admin-settings-help-cap-views-attachments = 用户是否拥有“附件”视图。邮件仍可通过搜索找到。
admin-settings-key-cap-search-operators = 搜索运算符
admin-settings-help-cap-search-operators = 用户是否可以使用 from:、subject: 或 has:attachment 等运算符搜索。
admin-settings-key-cap-saved-searches = 已保存的搜索
admin-settings-help-cap-saved-searches = 用户是否可以保存搜索。
admin-settings-key-cap-vacation = 休假自动回复
admin-settings-help-cap-vacation = 用户是否可以设置自动休假回复。
admin-settings-key-cap-labels = 标签
admin-settings-help-cap-labels = 用户是否可以为邮件添加标签。
admin-settings-key-cap-snooze = 稍后提醒
admin-settings-help-cap-snooze = 用户是否可以将邮件设为稍后提醒。仍可结束稍后提醒。
admin-settings-key-cap-follow-up = 跟进提醒
admin-settings-help-cap-follow-up = 用户是否可以为邮件设置跟进提醒。仍可清除提醒。
admin-settings-key-cap-masked = 隐藏地址
admin-settings-help-cap-masked = 用户是否可以创建和管理隐藏地址。
admin-settings-key-cap-directory = 目录搜索
admin-settings-help-cap-directory = 用户是否可以搜索组织目录。
admin-settings-key-cap-mcp = MCP 访问
admin-settings-help-cap-mcp = 用户的密钥和令牌是否可以使用 MCP 端点。
admin-settings-key-cap-appearance-custom = 外观自定义
admin-settings-help-cap-appearance-custom = 用户是否可以更改自己的主题、强调色、自定义颜色和徽章颜色、密度及字体。关闭则保持管理员设定的外观。
admin-settings-key-cap-appearance-custom-code = 自定义 CSS 和脚本
admin-settings-help-cap-appearance-custom-code = 用户是否可以保存自己的 CSS 和外观脚本。外观中的部署开关仍然优先于此项。
admin-settings-key-cap-avatars = 邮箱头像
admin-settings-help-cap-avatars = 用户是否可以更改邮箱头像。
admin-settings-key-cap-shortcuts = 自定义键盘快捷键
admin-settings-help-cap-shortcuts = 用户是否可以更改键盘快捷键。默认快捷键仍然有效。
admin-settings-key-cap-analytics = 产品分析
admin-settings-help-cap-analytics = 是否向用户征求分析同意，以及用户是否可以发送使用事件。
admin-settings-key-cap-apps-native = 桌面和移动应用
admin-settings-help-cap-apps-native = 用户是否可以使用桌面和移动应用。
admin-settings-key-access-protocols = 邮件访问协议
admin-settings-help-access-protocols = 这些用户可以使用哪些邮件服务器协议登录：imap、pop3、ews。通过 JMAP 服务器登录则遵循该服务器。
admin-settings-key-limit-linked-mailboxes = 每位用户的关联邮箱数
admin-settings-help-limit-linked-mailboxes = 用户可以关联的邮箱数量，低于服务器上限。0 表示只适用服务器上限。
admin-settings-key-login-show-account-help-notice = 在登录页显示账户帮助提示
admin-settings-help-login-show-account-help-notice = 在登录失败后显示一段简短的说明。默认隐藏；该提示不会透露账户是否存在或已被停用。
admin-settings-key-appearance-leetspeak = Leetspeak 文字样式
admin-settings-help-appearance-leetspeak = 以 leetspeak 显示界面标签和只读的邮件文字。原始邮件和可编辑的文字保持不变。除非此设置被锁定，用户可以选择正常文字。
admin-settings-key-mail-cache-enabled = 启用会话缓存
admin-settings-help-mail-cache-enabled = 将最近访问的文件夹和邮件保存在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-mailboxes = 缓存文件夹邮件列表
admin-settings-help-mail-cache-mailboxes = 将最近访问的文件夹和邮件保存在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-messages = 缓存已打开的邮件
admin-settings-help-mail-cache-messages = 将最近访问的文件夹和邮件保存在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-fresh-seconds = 新鲜度间隔（秒）
admin-settings-help-mail-cache-fresh-seconds = 将最近访问的文件夹和邮件保存在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-max-age-seconds = 缓存最长时效（秒）
admin-settings-help-mail-cache-max-age-seconds = 将最近访问的文件夹和邮件保存在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-max-entries = 缓存条目上限
admin-settings-help-mail-cache-max-entries = 将最近访问的文件夹和邮件保存在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-max-bytes = 内存预算（字节）
admin-settings-help-mail-cache-max-bytes = 将最近访问的文件夹和邮件保存在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-prefetch-enabled = 启用预测性预加载
admin-settings-help-mail-cache-prefetch-enabled = 将最近访问的文件夹和邮件保存在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-hover-enabled = 指向文件夹时预加载
admin-settings-help-mail-cache-hover-enabled = 将最近访问的文件夹和邮件保存在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-focus-enabled = 文件夹获得键盘焦点时预加载
admin-settings-help-mail-cache-focus-enabled = 将最近访问的文件夹和邮件保存在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-dwell-ms = 指针和焦点延迟（毫秒）
admin-settings-help-mail-cache-dwell-ms = 将最近访问的文件夹和邮件保存在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-idle-enabled = 空闲时预加载文件夹
admin-settings-help-mail-cache-idle-enabled = 将最近访问的文件夹和邮件保存在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-idle-delay-ms = 空闲延迟（毫秒）
admin-settings-help-mail-cache-idle-delay-ms = 将最近访问的文件夹和邮件保存在此标签页的内存中。缓存内容会在后台刷新。关闭标签页或退出登录会将其清除。预加载绝不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-max-concurrency = 同时预加载的上限

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-cache-max-concurrency = 在此标签页的内存中保留最近访问过的文件夹和邮件。缓存内容会在后台刷新。关闭标签页或退出登录后会清除。预加载不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-max-queued = 预加载队列上限
admin-settings-help-mail-cache-max-queued = 在此标签页的内存中保留最近访问过的文件夹和邮件。缓存内容会在后台刷新。关闭标签页或退出登录后会清除。预加载不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-preload-messages = 允许预先加载邮件正文
admin-settings-help-mail-cache-preload-messages = 在此标签页的内存中保留最近访问过的文件夹和邮件。缓存内容会在后台刷新。关闭标签页或退出登录后会清除。预加载不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-excluded-folders = 不进行预加载的文件夹
admin-settings-help-mail-cache-excluded-folders = 在此标签页的内存中保留最近访问过的文件夹和邮件。缓存内容会在后台刷新。关闭标签页或退出登录后会清除。预加载不会将邮件标为已读，也不会加载远程图片。
admin-settings-key-mail-cache-refresh-on-open = 打开时刷新缓存内容
admin-settings-help-mail-cache-refresh-on-open = 打开时刷新缓存内容
admin-settings-key-mail-templates-enabled = 回复模板
admin-settings-help-mail-templates-enabled = 在写信窗口中启用个人和共享的纯文本回复模板。除非此设置被锁定，否则用户可以选择不使用。
admin-settings-key-mail-templates-allow-personal = 允许个人模板
admin-settings-help-mail-templates-allow-personal = 允许用户创建自己的回复模板。共享模板由管理员单独管理。
admin-settings-key-mail-templates-shared = 共享回复模板
admin-settings-help-mail-templates-shared = 管理此范围内可重复使用的纯文本回复。模板不会设置收件人，也不会发送邮件。
admin-settings-key-mail-templates-personal = 个人回复模板
admin-settings-help-mail-templates-personal = 个人可重复使用的纯文本回复。创建或编辑需要启用回复模板和个人模板。
admin-settings-key-mail-compose-window = 写信窗口
admin-settings-help-mail-compose-window = 在工作区标签页或单独窗口中打开新草稿。弹出窗口被拦截时会改用标签页。
admin-settings-key-mail-reply-copy-to-original = 将回复保留在原文件夹
admin-settings-help-mail-reply-copy-to-original = 在支持的情况下，将已发送回复的副本保存到原邮件所在的文件夹。
admin-settings-key-mail-compose-check-invalid-recipients = 发送前检查无效收件人
admin-settings-help-mail-compose-check-invalid-recipients = 邮件发送到无效地址时的处理方式：允许、警告、警告并记住选择，或拒绝。
admin-settings-key-mail-compose-check-missing-attachment = 发送前检查是否遗漏附件
admin-settings-help-mail-compose-check-missing-attachment = 正文提到附件但未添加附件时的处理方式：允许、警告、警告并记住选择，或拒绝。
admin-settings-key-mail-compose-check-empty-subject = 发送前检查主题是否为空
admin-settings-help-mail-compose-check-empty-subject = 邮件没有主题时的处理方式：允许、警告、警告并记住选择，或拒绝。
admin-settings-key-mail-compose-auto-chip-recipients = 自动完成输入的收件人
admin-settings-help-mail-compose-auto-chip-recipients = 输入的地址后面跟空格、逗号或分号时，将其转换为收件人标签。
admin-settings-key-mail-compose-header-collapse = 写信窗口头部
admin-settings-help-mail-compose-header-collapse = 写信窗口中收件人、抄送、密送和主题各行的显示方式：展开、折叠为单行摘要，或自动（撰写正文时折叠，聚焦某个字段时展开）。除非此项被锁定，否则用户可以自行折叠或展开。
admin-settings-key-mail-compose-recipient-separator = 收件人分隔符
admin-settings-help-mail-compose-recipient-separator = 在完成一个收件人后插入的分隔符：逗号或分号。
admin-settings-key-mail-reply-from-mode = 回复和转发使用的身份
admin-settings-help-mail-reply-from-mode = 回复或转发时使用哪个身份发送：原邮件的收件身份、默认身份、选择器，或当前所选身份。
admin-settings-key-mail-forward-mode = 默认转发格式
admin-settings-help-mail-forward-mode = 以内联文本转发，或将原邮件作为 .eml 文件附上。
admin-settings-key-mail-show-go-to-top = 显示“回到顶部”按钮
admin-settings-help-mail-show-go-to-top = 在邮件列表中显示“回到顶部”按钮，点击后滚动回第一封邮件。默认关闭。
admin-settings-key-mail-show-create-folder-button = 显示“新建文件夹”按钮
admin-settings-help-mail-show-create-folder-button = 在侧边栏的“写邮件”旁显示仅含图标的“新建文件夹”按钮。默认关闭。
admin-settings-key-security-protection-user-agent-mode = User-Agent 观察模式
admin-settings-help-security-protection-user-agent-mode = 默认为强制：启用首次请求拦截机器人后，所选的已识别机器人请求头会被立即拦截。观察模式只记录判定而不拒绝；关闭则停用此维度。请求头中的声明不是经过验证的身份。
admin-settings-key-security-protection-user-agent-block-bots = 首次请求时拦截已识别的机器人
admin-settings-help-security-protection-user-agent-block-bots = 默认为开启。在强制模式下，所选的搜索、AI、自动化和自定义机器人请求头会在首次请求时触发临时 HTTP 429 拦截。观察模式只记录；关闭则恢复之前的请求额度。缺失或不常见的请求头、范围和例外保持原有行为。
admin-settings-key-security-protection-user-agent-categories = User-Agent 类别
admin-settings-help-security-protection-user-agent-categories = 配置可选 User-Agent 观察的这一部分。客户端提供的 User-Agent 文本只是弱信号，不能证明身份或滥用行为。
admin-settings-key-security-protection-user-agent-weak-requests = 弱信号请求阈值
admin-settings-help-security-protection-user-agent-weak-requests = 配置可选 User-Agent 观察的这一部分。客户端提供的 User-Agent 文本只是弱信号，不能证明身份或滥用行为。
admin-settings-key-security-protection-user-agent-crawler-requests = 爬虫请求阈值
admin-settings-help-security-protection-user-agent-crawler-requests = 配置可选 User-Agent 观察的这一部分。客户端提供的 User-Agent 文本只是弱信号，不能证明身份或滥用行为。
admin-settings-key-security-protection-user-agent-window-secs = User-Agent 观察窗口（秒）
admin-settings-help-security-protection-user-agent-window-secs = 配置可选 User-Agent 观察的这一部分。客户端提供的 User-Agent 文本只是弱信号，不能证明身份或滥用行为。
admin-settings-key-security-protection-user-agent-max-action = User-Agent 最高响应级别
admin-settings-help-security-protection-user-agent-max-action = 针对弱信号或重复请求证据的最高升级级别，默认为限流。此上限不限制对所选已识别机器人请求头的首次请求拦截。
admin-settings-key-security-protection-user-agent-scope = User-Agent 策略范围
admin-settings-help-security-protection-user-agent-scope = 配置可选 User-Agent 观察的这一部分。客户端提供的 User-Agent 文本只是弱信号，不能证明身份或滥用行为。
admin-settings-key-security-protection-user-agent-adaptive = 自适应 User-Agent 观察
admin-settings-help-security-protection-user-agent-adaptive = 配置可选 User-Agent 观察的这一部分。客户端提供的 User-Agent 文本只是弱信号，不能证明身份或滥用行为。
admin-settings-key-security-protection-user-agent-patterns = User-Agent 模式
admin-settings-help-security-protection-user-agent-patterns = 配置可选 User-Agent 观察的这一部分。客户端提供的 User-Agent 文本只是弱信号，不能证明身份或滥用行为。
admin-settings-key-security-protection-user-agent-exceptions = User-Agent 例外
admin-settings-help-security-protection-user-agent-exceptions = 配置可选 User-Agent 观察的这一部分。客户端提供的 User-Agent 文本只是弱信号，不能证明身份或滥用行为。
admin-settings-key-mail-reader-unsafe-html-mode = 可能不安全的邮件 HTML
admin-settings-help-mail-reader-unsafe-html-mode = 默认为“询问”，即为每封邮件提供经过净化的 HTML 按钮。“严格”只显示安全文本；“净化”自动显示清理后的 HTML。活动内容和自动加载的远程资源仍会被拦截。锁定后强制执行此策略。
admin-settings-key-reader-attachments-density = 附件密度
admin-settings-help-reader-attachments-density = 默认为紧凑的文件按钮，也可选择舒适的预览。此项控制邮件附件展开时的呈现方式。
admin-settings-key-cap-compose-open-tracking = 允许邮件打开跟踪
admin-settings-help-cap-compose-open-tracking = 此账户是否可以记录邮件被打开的情况。需要已配置的公开 URL；被停用的账户无法创建或绑定跟踪令牌。
admin-settings-key-mail-open-tracking = 默认跟踪打开情况
admin-settings-help-mail-open-tracking = 符合条件的格式化邮件的默认逐封跟踪选项。除非被锁定，否则用户可以覆盖；停用跟踪功能始终优先。
admin-settings-key-cap-reading-original-html = 允许渲染原始 HTML
admin-settings-help-cap-reading-original-html = 用户是否可以为单封邮件选择查看原始 HTML。默认的净化渲染器和隔离框架安全措施保持不变。
admin-settings-key-mail-reader-allow-original-html = 提供原始 HTML 视图
admin-settings-help-mail-reader-allow-original-html = 在阅读功能允许时，显示针对单封邮件使用原始 HTML 的选项。默认关闭；不会改变默认的邮件渲染器。
admin-settings-key-privacy-tracker-override-allowed = 提供逐封邮件的跟踪器授权
admin-settings-help-privacy-tracker-override-allowed = 在邮件隐私栏中显示“允许跟踪器”操作。默认可用，但没有明确授权时绝不会生效。被锁定的跟踪器拦截策略仍会禁止覆盖。
admin-settings-key-privacy-max-concurrent-global = 全局并行图片获取数
admin-settings-help-privacy-max-concurrent-global = 此服务器进程中活动图片网络请求的最大数量。范围 1–256；默认 128。邮件和账户限制仍然适用。
admin-settings-key-privacy-max-concurrent-pipeline = 并发图片处理流水线数
admin-settings-help-privacy-max-concurrent-pipeline = 同时允许进入获取、解码和缓存发布流程的最大图片数。范围 1–256；默认 128。网络和解码器上限另行执行。
admin-settings-key-privacy-max-concurrent-decodes = 并行图片解码器数
admin-settings-help-privacy-max-concurrent-decodes = 同时运行的图片解码最大数量。范围 1–16；默认 10。数值越高，占用的 CPU 和内存越多。
admin-settings-key-privacy-max-queued-requests = 等待中的图片请求数
admin-settings-help-privacy-max-queued-requests = 等待图片网络或处理能力的请求最大数量。范围 16–16384；默认 2048。溢出或队列过期后仍可重试。
admin-settings-key-privacy-rate-burst = 图片请求突发额度
admin-settings-help-privacy-rate-burst = 每个账户初始及最大的获取令牌数，按配置的图片请求速率补充。范围 1–12000；默认 1024。缓存命中的成功请求不消耗令牌。
admin-settings-key-privacy-image-rate-mode = 图片请求速率调节方式
admin-settings-help-privacy-image-rate-mode = 自动模式会根据副本容量、队列负载和邮件的图片数量，在下方的下限和上限内调整每个账户的突发额度和补充速度。手动模式则严格使用突发额度和请求速率设置。默认为自动。超出额度的请求会等待令牌，直到获取队列截止时间；会先检查身份验证和授权。
admin-settings-key-privacy-image-rate-burst-floor = 自动突发额度下限
admin-settings-help-privacy-image-rate-burst-floor = 自动速率模式下每个账户的最低突发额度。范围 1–12000；默认 64。
admin-settings-key-privacy-image-rate-burst-ceiling = 自动突发额度上限
admin-settings-help-privacy-image-rate-burst-ceiling = 自动速率模式下每个账户的最高突发额度。范围 1–12000；默认 2048。
admin-settings-key-privacy-image-rate-refill-floor = 自动补充速度下限
admin-settings-help-privacy-image-rate-refill-floor = 自动速率模式下每个账户的最低补充速度，单位为每分钟获取令牌数。范围 10–60000；默认 300。
admin-settings-key-privacy-image-rate-refill-ceiling = 自动补充速度上限
admin-settings-help-privacy-image-rate-refill-ceiling = 自动速率模式下每个账户的最高补充速度，单位为每分钟获取令牌数。范围 10–60000；默认 6000。
admin-settings-key-privacy-image-cache-mode = 图片缓存大小调节方式
admin-settings-help-privacy-image-cache-mode = 自动模式根据扣除解码器和流水线预留后剩余的内存确定缓存大小，并在命中率高且内存充足时延长缓存时间。手动模式则严格使用缓存大小、条目数和存活时间设置。默认为自动。
admin-settings-key-privacy-image-cache-bytes-floor = 自动缓存大小下限
admin-settings-help-privacy-image-cache-bytes-floor = 自动模式下图片缓存的最小字节数。范围 0–1073741824；默认 16777216。
admin-settings-key-privacy-image-cache-bytes-ceiling = 自动缓存大小上限
admin-settings-help-privacy-image-cache-bytes-ceiling = 自动模式下图片缓存的最大字节数。范围 0–1073741824；默认 268435456。
admin-settings-key-privacy-cache-entries = 图片缓存条目上限
admin-settings-help-privacy-cache-entries = 缓存图片的最大数量；手动模式下为固定数量，自动模式下为上限。范围 16–65536；默认 4096。
admin-settings-key-privacy-cache-entry-bytes = 单张缓存图片最大大小
admin-settings-help-privacy-cache-entry-bytes = 缓存中保留的单张重新编码图片的最大字节数。更大的图片会被提供但不会被保留。范围 65536–16777216；默认 4194304。
admin-settings-key-privacy-image-cache-ttl-floor-secs = 自动缓存存活时间下限
admin-settings-help-privacy-image-cache-ttl-floor-secs = 自动模式下最短的缓存存活时间（秒），在命中率低或内存紧张时使用。范围 0–86400；默认 300。
admin-settings-key-privacy-image-cache-ttl-ceiling-secs = 自动缓存存活时间上限
admin-settings-help-privacy-image-cache-ttl-ceiling-secs = 自动模式下最长的缓存存活时间（秒），在命中率高且内存充足时使用。范围 0–86400；默认 21600。
admin-settings-key-privacy-cache-ttl-secs = 成功图片的缓存存活时间
admin-settings-help-privacy-cache-ttl-secs = 解码后的图片在此进程中可重复使用的秒数。范围 0–86400；默认 3600。设为零则停用成功图片的复用；始终会先检查授权。
admin-settings-key-privacy-image-latency-tracking = 图片代理速度跟踪
admin-settings-help-privacy-image-latency-tracking = 在内存中按固定时间桶计数，记录每张代理图片所用的时间（中位数、p95、p99，以及等待、获取和解码所花的时间）。不存储 URL、主机或账户。默认开启；历史记录在重启时重置，且按服务器进程分别记录。关闭后停止新的记录；“速度”部分仍会显示已记录的内容。
admin-settings-key-privacy-negative-cache-ttl-secs = 失败图片的缓存存活时间
admin-settings-help-privacy-negative-cache-ttl-secs = 符合条件的上游失败结果的记忆秒数，以避免重复请求。范围 0–300；默认 10。设为零则停用失败结果的复用。明确重试会清除受影响的失败记录。
admin-settings-key-privacy-image-retry-attempts = 图片自动重试次数
admin-settings-help-privacy-image-retry-attempts = 一次可见邮件加载中，对临时性图片失败的最大重试次数。范围 0–8；默认 3。设为零则停用自动重试，但保留手动重试。
admin-settings-key-privacy-image-retry-delay-ms = 图片自动重试间隔
admin-settings-help-privacy-image-retry-delay-ms = 当前帧稳定后，两次自动图片重试之间的毫秒数。范围 500–60000；默认 5000。永久性拒绝不会自动重试。
admin-settings-key-privacy-image-decode-timeout-ms = 图片解码超时
admin-settings-help-privacy-image-decode-timeout-ms = 获取完成后，单次沙盒图片解码允许的实际耗时（毫秒）。范围 1000–60000；默认 10000。处理大图片时可调高；网络超时另行设置。
admin-settings-key-privacy-image-concurrency-mode = 图片并发配置
admin-settings-help-privacy-image-concurrency-mode = 自动模式会根据 CPU 容量、容器内存和每封邮件已批准的图片数量进行调整。“低资源”“均衡”和“高吞吐”同样会自适应，但安全上限不同。手动模式直接使用数值上限。应用某个配置只会更改此设置。
admin-settings-cat-image-proxy = 图片代理
admin-settings-key-privacy-image-ram-max-mi-b = 内存层上限
admin-settings-help-privacy-image-ram-max-mi-b = 内存图片层的上限（MiB），在自动或手动缓存大小之上生效。设为零则不设上限。请保持较小，让磁盘层承担主要容量。范围 0–1048576；默认 0。
admin-settings-key-privacy-image-disk-enabled = 图片磁盘层
admin-settings-help-privacy-image-disk-enabled = 在内存层之后，将已重新编码的图片保存到磁盘。默认关闭；目录缺失、已满或只读时，缓存会静默地仅在内存中继续运行。仅存储沙盒输出，并使用哈希文件名。
admin-settings-key-privacy-image-disk-path = 图片磁盘目录
admin-settings-help-privacy-image-disk-path = 磁盘层使用的绝对目录。留空则使用数据库文件旁的 image-cache 文件夹。根文件系统只读时，请在此处挂载可写卷。相对路径和 .. 会被拒绝，并使该层保持关闭。
admin-settings-key-privacy-image-disk-mode = 图片磁盘大小调节方式
admin-settings-help-privacy-image-disk-mode = 自动模式使用此缓存可用空间（剩余空间加上已占用空间）的十分之一，介于下限和上限之间，且不超过该空间的一半。手动模式则严格使用下方的最大大小。默认为自动。
admin-settings-key-privacy-image-disk-max-mi-b = 图片磁盘大小（手动）
admin-settings-help-privacy-image-disk-max-mi-b = 手动模式下磁盘层的大小（MiB）；读数在两种模式下都显示实际生效的大小。范围 1–1048576；默认 512。
admin-settings-key-privacy-image-disk-floor-mi-b = 自动磁盘大小下限
admin-settings-help-privacy-image-disk-floor-mi-b = 自动模式下磁盘层的最小大小（MiB）；剩余空间未知时也使用此值。范围 0–1048576；默认 64。
admin-settings-key-privacy-image-disk-ceiling-mi-b = 自动磁盘大小上限
admin-settings-help-privacy-image-disk-ceiling-mi-b = 自动模式下磁盘层的最大大小（MiB）。范围 0–1048576；默认 4096。

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-privacy-image-disk-entries = 磁盘条目上限
admin-settings-help-privacy-image-disk-entries = 磁盘上保留的最大图片数。每个条目还会在索引中占用固定大小的内存，计入内存层。范围 16–1048576；默认 16384。
admin-settings-key-privacy-image-disk-entry-bytes = 单张磁盘图片最大大小
admin-settings-help-privacy-image-disk-entry-bytes = 写入磁盘的单张重新编码图片的最大字节数。超过内存条目上限但不超过此值的图片会直接写入磁盘。范围 65536–67108864；默认 8388608。
admin-settings-key-privacy-image-disk-min-hits = 写入磁盘前的命中次数
admin-settings-help-privacy-image-disk-min-hits = 内存条目被逐出时，需要达到多少次命中才会写入磁盘而不是被丢弃。设为零则会写入所有得分通过降级阈值的被逐出条目。范围 0–100；默认 1。
admin-settings-key-privacy-image-disk-write-queue-mi-b = 磁盘写入队列
admin-settings-help-privacy-image-disk-write-queue-mi-b = 可等待后台磁盘写入器的图片数据量（MiB）。计入内存层；超出后，被逐出的图片会被丢弃而不是排队。范围 1–256；默认 8。
admin-settings-key-privacy-image-ttl-step-secs = 每次命中延长的存活时间
admin-settings-help-privacy-image-ttl-step-secs = 命中时为图片剩余存活时间增加的秒数，乘以 1 加上其衰减命中次数的对数，因此常用图片存活更久。设为零则所有图片都保持基础存活时间。范围 0–86400；默认 600。
admin-settings-key-privacy-image-ttl-max-secs = 延长后的最长存活时间
admin-settings-help-privacy-image-ttl-max-secs = 命中可累积出的最长剩余存活时间（秒）。范围 60–2592000；默认 86400。
admin-settings-key-privacy-image-max-age-secs = 图片最长保留时间
admin-settings-help-privacy-image-max-age-secs = 无论命中情况如何，超过此时长（秒）的图片都会重新获取。获取路径不保留上游的 ETag 或 Last-Modified，因此没有条件性重新验证，只是重新获取图片。范围 3600–7776000；默认 604800。
admin-settings-key-privacy-image-score-half-life-secs = 得分半衰期
admin-settings-help-privacy-image-score-half-life-secs = 在决定内存、磁盘和逐出的优先级得分中，图片的命中次数和新近度减半所需的秒数。范围 60–604800；默认 3600。
admin-settings-key-privacy-image-score-demote-milli = 降级得分
admin-settings-help-privacy-image-score-demote-milli = 以千分之一为单位的得分，低于此值的内存图片会被降级到磁盘或丢弃。升级阈值为此值加上滞后量。范围 0–10000；默认 300。
admin-settings-key-privacy-image-score-hysteresis-percent = 升级滞后量
admin-settings-help-privacy-image-score-hysteresis-percent = 升级阈值高于降级阈值的百分比。得分介于两者之间时，图片保持原位。范围 0–500；默认 50。
admin-settings-key-privacy-image-context-open-percent = 已打开邮件的权重
admin-settings-help-privacy-image-context-open-percent = 为某封邮件请求的图片的得分权重（百分比）。邮件的图片在空闲十分钟后算作“最近”。范围 100–1000；默认 200。
admin-settings-key-privacy-image-context-recent-percent = 最近邮件的权重
admin-settings-help-privacy-image-context-recent-percent = 最近使用过的图片的得分权重（百分比）。未指明邮件的请求权重为 100。范围 100–1000；默认 150。
admin-settings-key-privacy-image-account-share-percent = 账户内存份额
admin-settings-help-privacy-image-account-share-percent = 在其他账户使用缓存时，单个账户的写入最多可占用内存层的比例；超出后，该账户会先逐出自己得分最低的图片。单个账户可以使用全部内存层。范围 1–100；默认 50。
admin-settings-key-privacy-image-sweep-secs = 缓存清扫间隔
admin-settings-help-privacy-image-sweep-secs = 后台清扫之间的间隔秒数，清扫会使图片过期、对一批有限条目重新评分、在压力下降级冷的内存图片，并重新检查磁盘剩余空间。范围 5–3600；默认 30。
admin-settings-key-privacy-image-snapshot-secs = 索引快照间隔
admin-settings-help-privacy-image-snapshot-secs = 磁盘索引原子快照之间的间隔秒数，使命中次数和延长的存活时间在重启后得以保留，而无需每次命中都写入。设为零则关闭快照；重启后将读取每个文件的头部。范围 0–86400；默认 300。
admin-settings-key-privacy-image-sweep-batch = 清扫批量大小
admin-settings-help-privacy-image-sweep-batch = 一次维护清扫可使其过期、重新评分或逐出的条目数。限制在持有缓存锁时所做的工作量。范围 16–8192；默认 256。
admin-image-proxy-title = 图片缓存读数
admin-image-proxy-unavailable = 图片缓存读数不可用。
admin-image-proxy-unknown = 未知
admin-image-proxy-ram = 内存层：已用 { $used }，预算 { $budget }（配置 { $configured }），{ $entries } 张图片，热 { $hot } 张，冷 { $cold } 张，命中率 { $hit }。
admin-image-proxy-disk-off = 磁盘层：已关闭。
admin-image-proxy-disk = 磁盘层：已用 { $bytes }，实际预算 { $budget }（{ $mode }；下限 { $floor }，上限 { $ceiling }，手动 { $manual }），{ $entries } 张图片，热 { $hot } 张，冷 { $cold } 张，命中率 { $hit }。
admin-image-proxy-disk-inactive = 磁盘层：已开启但未生效。{ $reason }
admin-image-proxy-moves = 每分钟升级次数：{ $perMin }。累计升级 { $promotions } 次，降级 { $demotions } 次，丢弃 { $drops } 次。索引条目：{ $index }。
admin-image-proxy-refresh = 刷新读数
admin-settings-key-security-trust-enabled = 信任层级
admin-settings-help-security-trust-enabled = 为每个请求划分层级：已登录、已知来源（该地址最近有成功登录）或未知。只会改变匿名请求的额度；凭据、第二重验证和锁定永远不会放宽。
admin-settings-key-security-trust-known-ttl-days = 已知来源有效期（天）
admin-settings-help-security-trust-known-ttl-days = 一次成功登录使其地址保持“已知”的时长。失败的尝试不计入。
admin-settings-key-security-trust-known-capacity = 保留的已知来源数
admin-settings-help-security-trust-known-capacity = 内存中保留的已知来源最大数量（每项包含加盐哈希、国家和账户数）。最旧的会最先被丢弃。
admin-settings-key-security-trust-persist = 重启后保留已知来源
admin-settings-help-security-trust-persist = 将已知来源表（带密钥的哈希、国家、最近出现时间、账户数）保存到设置存储，并在启动时加载。超出有效期的条目会被丢弃。
admin-settings-key-security-trust-persist-interval-secs = 已知来源保存间隔（秒）
admin-settings-help-security-trust-persist-interval-secs = 两次保存之间的秒数，范围 30 到 3600。仅在表发生变化时保存，并在正常关闭时再保存一次。
admin-settings-key-security-trust-unknown-percent = 未知来源额度（%）
admin-settings-help-security-trust-unknown-percent = 未知来源的匿名请求额度，占已配置限制的百分比。100 保持基准；越低越严格。
admin-settings-key-security-trust-known-percent = 已知来源额度（%）
admin-settings-help-security-trust-known-percent = 已知来源的匿名请求额度。200 会使请求预算翻倍，并将登录冷却时间和挑战难度减半。绝不会绕过凭据、第二重验证或锁定。
admin-settings-key-security-trust-ua-missing = 机器人：空 User-Agent
admin-settings-help-security-trust-ua-missing = 对 User-Agent 为空的匿名请求采取的操作：允许、限制或拦截。已登录的会话不受影响。
admin-settings-key-security-trust-ua-uncommon = 机器人：无法识别的 User-Agent
admin-settings-help-security-trust-ua-uncommon = 对 User-Agent 无法识别的匿名请求采取的操作：允许、限制或拦截。
admin-settings-key-security-trust-ua-automation = 机器人：脚本和自动化
admin-settings-help-security-trust-ua-automation = 对自称为脚本、HTTP 库、无头浏览器或通用机器人的匿名请求采取的操作：允许、限制或拦截。
admin-settings-key-security-trust-ua-search = 机器人：搜索爬虫
admin-settings-help-security-trust-ua-search = 对自称为搜索引擎爬虫的匿名请求采取的操作：允许、限制或拦截。
admin-settings-key-security-trust-ua-ai-training = 机器人：AI 训练爬虫
admin-settings-help-security-trust-ua-ai-training = 对自称为 GPTBot、ClaudeBot、CCBot、Amazonbot、Bytespider、Google-Extended 或 Applebot-Extended 的匿名请求采取的操作：允许、限制或拦截。自称只是请求头，并非经过验证的身份。
admin-settings-key-security-trust-ua-ai-search = 机器人：AI 搜索爬虫
admin-settings-help-security-trust-ua-ai-search = 对自称为 OAI-SearchBot、PerplexityBot 或 Claude-SearchBot 的匿名请求采取的操作：允许、限制或拦截。
admin-settings-key-security-trust-ua-ai-user = 机器人：AI 用户代取工具
admin-settings-help-security-trust-ua-ai-user = 对自称为 ChatGPT-User 或 Claude-User 的匿名请求采取的操作：允许、限制或拦截。
admin-settings-key-security-trust-ua-custom = 机器人：自定义模式
admin-settings-help-security-trust-ua-custom = 对匹配自定义 User-Agent 模式的匿名请求采取的操作：允许、限制或拦截。
admin-settings-key-security-trust-admin-geo-enabled = 拦截来自未知国家的管理探测
admin-settings-help-security-trust-admin-geo-enabled = 拦截来自最近没有成功登录记录的国家、并探测管理控制台的来源。需要可读取的国家数据库和管理探测防护。设置 MW_ADMIN_GEO_ENABLED=false 可在恢复时将其关闭。
admin-settings-key-security-trust-admin-geo-countries = 始终视为已知的国家（管理拦截）
admin-settings-help-security-trust-admin-geo-countries = 除最近成功登录所在的国家外，始终视为已知的大写 ISO 国家代码。
admin-settings-key-security-trust-admin-geo-lookup-failure = 无国家信息的来源
admin-settings-help-security-trust-admin-geo-lookup-failure = allow 不拦截并记录该情况；unknown 将未检测到国家的来源视为未知国家。
admin-settings-key-security-trust-admin-geo-threshold = 拦截前的探测次数
admin-settings-help-security-trust-admin-geo-threshold = 窗口期内来自未知国家的管理探测达到多少次后开始拦截。
admin-settings-key-security-trust-admin-geo-window-secs = 探测窗口（秒）
admin-settings-help-security-trust-admin-geo-window-secs = 探测阈值所用的窗口。
admin-settings-key-security-trust-admin-geo-block-on-first = 首次探测即拦截
admin-settings-help-security-trust-admin-geo-block-on-first = 来自未知国家的首次探测即拦截。这会增加将出差中的管理员锁在门外的可能性。
admin-settings-key-security-trust-admin-geo-block-secs = 首次拦截时长（秒）
admin-settings-help-security-trust-admin-geo-block-secs = 首次拦截的时长。
admin-settings-key-security-trust-admin-geo-second-block-secs = 第二次拦截时长（秒）
admin-settings-help-security-trust-admin-geo-second-block-secs = 同一来源在重复期内第二次被拦截的时长。
admin-settings-key-security-trust-admin-geo-max-block-secs = 重复拦截时长（秒）
admin-settings-help-security-trust-admin-geo-max-block-secs = 重复期内第三次及之后每次拦截的时长。
admin-settings-key-security-trust-admin-geo-repeat-days = 重复期（天）
admin-settings-help-security-trust-admin-geo-repeat-days = 距上次拦截不超过此天数的拦截视为重复，并使用下一档更长的时长。
admin-settings-key-security-signin-account-geo-mode = 按账户的登录国家策略
admin-settings-help-security-signin-account-geo-mode = 默认关闭。拦截来自该账户以前未使用过的国家、或不在其允许列表内的任何国家的登录。允许列表中的 IP 和国家始终放行。仅适用于新的网页邮箱登录，不影响现有会话或协议客户端。国家检测使用 IP 地理位置数据库，可能有误或已过时；VPN、移动运营商、代理和出行可能会拦截合法登录，也可能放行境外登录。请将可信的 IP 或国家加入允许列表。
admin-settings-key-security-signin-account-geo-allow-countries = 账户允许的国家
admin-settings-help-security-signin-account-geo-allow-countries = 该账户始终允许的大写 ISO 国家代码。国家检测使用 IP 地理位置数据库，可能有误或已过时；VPN、移动运营商、代理和出行可能会拦截合法登录，也可能放行境外登录。请将可信的 IP 或国家加入允许列表。
admin-settings-key-security-signin-account-geo-allow-ips = 账户允许的 IP
admin-settings-help-security-signin-account-geo-allow-ips = 该账户始终允许的 IP 地址或 CIDR 网络。单个 IPv6 地址指其 /64 网段。
admin-settings-key-security-signin-account-geo-global-allow-countries = 所有账户允许的国家
admin-settings-help-security-signin-account-geo-global-allow-countries = 所有账户都允许的大写 ISO 国家代码，会加入各账户自己的列表。
admin-settings-key-security-signin-account-geo-global-allow-ips = 所有账户允许的 IP
admin-settings-help-security-signin-account-geo-global-allow-ips = 所有账户都允许的 IP 地址或 CIDR 网络（办公室或 VPN），会加入各账户自己的列表。
admin-settings-key-security-signin-account-geo-lookup-failure = 无国家信息的来源（账户策略）
admin-settings-help-security-signin-account-geo-lookup-failure = 来源未检测到国家时：允许登录并记录，或拒绝登录。
admin-settings-key-security-signin-account-geo-history-days = 账户国家历史（天）
admin-settings-help-security-signin-account-geo-history-days = 为异常国家策略，某个国家在账户自身登录历史中保留的天数。它与身份验证日志分开保存，因此缩短日志不会缩短它。
admin-settings-key-security-geo-no-database = 无国家数据库：默认行为
admin-settings-help-security-geo-no-database = 未加载数据库时，国家相关功能的处理方式：允许、将来源视为未知国家，或拦截（拦截仅适用于登录）。功能自身的设置也会生效，取其中更严格的。
admin-settings-key-security-geo-not-found = 数据库中没有该地址：默认行为
admin-settings-help-security-geo-not-found = 已加载数据库但其中没有该地址的条目时，国家相关功能的处理方式。私有、回环和链路本地地址属于本地网络，永远不受影响。
admin-settings-key-security-signin-geo-no-database = 无数据库时的实例登录策略
admin-settings-help-security-signin-geo-no-database = 实例登录国家策略已开启但未加载数据库时：拦截登录（失败即关闭），或不经检查直接允许。本地网络始终允许。
admin-settings-key-security-geo-session-lookup-failure = 无国家信息的会话提醒
admin-settings-help-security-geo-session-lookup-failure = 无法确定已登录请求所在国家时：忽略，或每个会话每天发出一次会话提醒。绝不会拦截或撤销会话。
admin-settings-key-security-protocol-framing-mode = 协议防护：请求分帧
admin-settings-help-security-protocol-framing-mode = 请求走私类问题：Content-Length 与 Transfer-Encoding 同时出现，Transfer-Encoding 不是恰好为 chunked（或出现在 HTTP/1.0 和 HTTP/2 上），Content-Length 非数字或相互冲突。强制模式返回 400 并关闭连接；仅检测模式只计数。路由前缀豁免永远不适用于此类问题。
admin-settings-key-security-protocol-host-mode = 协议防护：Host 语法
admin-settings-help-security-protocol-host-mode = Host 头语法：HTTP/1.1 缺失、为空、重复，或包含用户信息、空白字符或路径字符。接受 IPv6 字面量、端口和末尾的点。强制模式返回 400 并关闭连接。
admin-settings-key-security-protocol-host-pinning-mode = 协议防护：Host 必须与公开名称一致
admin-settings-help-security-protocol-host-pinning-mode = Host 与 MW_PUBLIC_URL 的主机或 allowedHosts 之一不匹配。两者都未指定主机时不起作用。默认为仅检测，因为代理和健康检查可能发送内部名称。
admin-settings-key-security-protocol-path-control-mode = 协议防护：路径中的控制字符
admin-settings-help-security-protocol-path-control-mode = 请求路径中的 NUL、控制字符或编码后的 CR/LF（%00、%0d、%0a）。强制模式返回 400。永远不会被路由前缀豁免。
admin-settings-key-security-protocol-target-mode = 协议防护：请求目标伎俩
admin-settings-help-security-protocol-target-mode = 对点和斜杠的双重或过长百分号编码、编码的点、点段、空路径段（//）、反斜杠、原始非 ASCII 字节，以及指向 Host 之外其他权限的绝对形式目标。
admin-settings-key-security-protocol-path-length-mode = 协议防护：路径和查询长度
admin-settings-help-security-protocol-path-length-mode = 路径或查询超过 maxPathBytes 或 maxQueryBytes。强制模式返回 414。
admin-settings-key-security-protocol-method-mode = 协议防护：HTTP 方法
admin-settings-help-security-protocol-method-mode = 方法不在 allowedMethods 之内（默认包括 TRACE、TRACK、CONNECT，以及未知或小写的方法）。强制模式返回 405。
admin-settings-key-security-protocol-duplicate-headers-mode = 协议防护：重复的单值请求头
admin-settings-help-security-protocol-duplicate-headers-mode = 本应只出现一次的请求头出现多次（Content-Type、Authorization、Origin、Referer、User-Agent、Sec-Fetch-*、条件和范围请求头），值相同的重复 Content-Length，或 HTTP/1.x 上重复的 Cookie 头。
admin-settings-key-security-protocol-forwarded-mode = 协议防护：来自不受信任对端的转发头
admin-settings-help-security-protocol-forwarded-mode = 由 MW_TRUSTED_PROXIES 之外的对端发送的 Forwarded、X-Forwarded-* 或 X-Real-IP。无论如何这些头都不会被信任；此项只记录这种尝试。仅在配置了可信代理时才计入自适应防护。
admin-settings-key-security-protocol-header-hygiene-mode = 协议防护：请求头规范性
admin-settings-help-security-protocol-header-hygiene-mode = 请求头数量、单个值或 Cookie 头超出配置上限；请求头含非 ASCII 字节；WebSocket 路由之外出现 Upgrade 或 Connection: upgrade;Connection 指向 Content-Length 或 Host 等请求头。强制模式对大小和数量问题返回 431，其余返回 400。
admin-settings-key-security-protocol-body-mode = 协议防护：请求体分帧
admin-settings-help-security-protocol-body-mode = GET 或 HEAD 带有请求体，Content-Length 超过 maxBodyBytes（413），Expect 不是 100-continue（417）或没有请求体。
admin-settings-key-security-protocol-max-path-bytes = 协议防护：最长路径（字节）
admin-settings-help-security-protocol-max-path-bytes = 路径长度类问题上报之前允许的最长请求路径。
admin-settings-key-security-protocol-max-query-bytes = 协议防护：最长查询（字节）
admin-settings-help-security-protocol-max-query-bytes = 路径长度类问题上报之前允许的最长查询字符串。
admin-settings-key-security-protocol-max-header-count = 协议防护：最多请求头数
admin-settings-help-security-protocol-max-header-count = 请求头规范类问题上报之前，单个请求允许的最多请求头字段数。监听器自身的硬性上限（MW_HTTP_MAX_HEADERS）仍然适用。
admin-settings-key-security-protocol-max-header-value-bytes = 协议防护：最长请求头值（字节）
admin-settings-help-security-protocol-max-header-value-bytes = 请求头规范类问题上报之前允许的最长单个请求头值。
admin-settings-key-security-protocol-max-cookie-bytes = 协议防护：最大 Cookie 头（字节）
admin-settings-help-security-protocol-max-cookie-bytes = 请求头规范类问题上报之前，Cookie 头合计允许的大小。
admin-settings-key-security-protocol-max-body-bytes = 协议防护：最大声明请求体（字节）
admin-settings-help-security-protocol-max-body-bytes = 请求体类检查不上报的最大 Content-Length。请将其保持在路由接受的最大上传或导入之上；路由前缀豁免可为某个路由调高它。
admin-settings-key-security-protocol-allowed-methods = 协议防护：允许的方法
admin-settings-help-security-protocol-allowed-methods = 方法类检查接受的方法。默认列出常规 HTTP 动词以及 WebDAV、CalDAV 和 CardDAV 动词；服务器目前未挂载任何入站 DAV 路由，因此这些请求会在路由器处得到 405。
admin-settings-key-security-protocol-allowed-hosts = 协议防护：额外接受的主机
admin-settings-help-security-protocol-allowed-hosts = 除 MW_PUBLIC_URL 的主机外，Host 固定检查额外接受的主机名，例如代理或健康检查使用的内部名称。端口和末尾的点会被忽略。
admin-settings-key-security-protocol-exempt-prefixes = 协议防护：豁免的路由前缀
admin-settings-help-security-protocol-exempt-prefixes = 不对其运行启发式检查的路由前缀（每个以 / 开头且长度超过一个字符），供确有需要的合法客户端使用，例如大文件上传路由。按完整路径段匹配。分帧和路径控制问题永远不会被豁免，/ 会被忽略。
admin-settings-key-security-health-mode = 健康检查端点访问
admin-settings-help-security-health-mode = 谁可以访问 /healthz 和 /readyz。internal-only：回环、私有、链路本地和唯一本地客户端地址。allowlist：以上加上下方的网络。open：任何人。其他人会得到与未知路径相同的 404。携带 MW_HEALTH_TOKEN 令牌的请求始终放行。客户端地址是经可信代理解析后的地址：在没有设置 MW_FORWARDED_MODE 和 MW_TRUSTED_PROXIES 的反向代理之后，每个请求都像来自代理的私有地址，因此整个互联网都会被放行。
admin-settings-key-security-health-networks = 健康检查端点网络
admin-settings-help-security-health-networks = 在 allowlist 模式下，除内部范围外还可访问 /healthz 和 /readyz 的 IP 地址或 CIDR 网络，例如外部可用性监控。
admin-settings-key-security-spray-enabled = 密码喷洒检测
admin-settings-help-security-spray-enabled = 统计同一来源登录失败时涉及的不同用户名数量。内存中只保留每个用户名的带密钥哈希。
admin-settings-key-security-spray-window-secs = 喷洒窗口（秒）

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-security-spray-window-secs = 按不同用户名统计同一来源登录失败的窗口。默认 900。
admin-settings-key-security-spray-challenge-at = 喷洒：挑战阈值
admin-settings-help-security-spray-challenge-at = 窗口内来自同一来源、登录失败的不同用户名数量达到此值时，对该来源强制要求工作量证明挑战。成功登录不计入，因此共用的办公室或校园地址不受影响。默认 10。
admin-settings-key-security-spray-pace-at = 喷洒：限速阈值
admin-settings-help-security-spray-pace-at = 登录失败的不同用户名数量达到此值时，会增加该来源两次尝试之间的等待时间。默认 15。
admin-settings-key-security-spray-lock-at = 喷洒：锁定阈值
admin-settings-help-security-spray-lock-at = 登录失败的不同用户名数量达到此值时，通过现有的登录锁定机制将该来源锁定一段锁定时长。默认 25。
admin-settings-key-security-spray-pace-secs = 喷洒：限速等待（秒）
admin-settings-help-security-spray-pace-secs = 来源达到或超过限速阈值后，两次尝试之间间隔的秒数。默认 10。
admin-settings-key-security-spray-lock-secs = 喷洒：锁定时长（秒）
admin-settings-help-security-spray-lock-secs = 临时锁定的时长。管理员解锁该来源时也会重置其计数。默认 900。
admin-settings-key-security-spray-known-multiplier = 喷洒：已知来源倍数
admin-settings-help-security-spray-known-multiplier = 对已知来源（最近有成功登录）的三个阈值所乘的倍数。默认 3。
admin-settings-key-security-spray-capacity = 喷洒：跟踪的来源数
admin-settings-help-security-spray-capacity = 内存中跟踪的最大来源数；最久未活跃的会最先被丢弃。默认 20000。
admin-settings-key-security-spike-enabled = 全实例突增检测
admin-settings-help-security-spike-enabled = 学习登录失败的基线，当整个实例远高于基线时进入警戒模式。它绝不会锁定实例。
admin-settings-key-security-spike-multiplier-percent = 突增：倍数(%)
admin-settings-help-security-spike-multiplier-percent = 当某个信号至少达到其已学习基线的这一百分比（400 即四倍）且高于其绝对下限时才计入。
admin-settings-key-security-spike-min-failures-per-min = 突增：每分钟最少失败数
admin-settings-help-security-spike-min-failures-per-min = 下限：每分钟登录失败数低于此值时，失败率信号绝不计入。默认 20。
admin-settings-key-security-spike-min-sources = 突增：最少失败来源数
admin-settings-help-security-spike-min-sources = 下限：五分钟区间内失败的不同来源数低于此值时，该信号绝不计入。默认 10。
admin-settings-key-security-spike-min-usernames = 突增：最少失败用户名数
admin-settings-help-security-spike-min-usernames = 下限：五分钟区间内失败的不同用户名数低于此值时，该信号绝不计入。默认 30。
admin-settings-key-security-spike-signals-required = 突增：所需信号数
admin-settings-help-security-spike-signals-required = 必须同时超过基线和下限的信号数量。分布式喷洒特征算作两个。默认 2。
admin-settings-key-security-spike-warmup-hours = 突增：预热（小时）
admin-settings-help-security-spike-warmup-hours = 在警戒模式可以启动之前需要观察的流量小时数。在此之前基线只学习。默认 48。
admin-settings-key-security-spike-adapt-hours = 突增：基线适应（小时）
admin-settings-help-security-spike-adapt-hours = 基线的时间常数。越长，吸收增长越慢。突增绝不会更新基线。默认 168。
admin-settings-key-security-spike-duration-secs = 突增：警戒模式时长（秒）
admin-settings-help-security-spike-duration-secs = 最后一次出现突增证据后，警戒模式持续的时间；只要信号仍高于其触发水平的一半，就会延长。默认 1800。
admin-settings-key-security-spike-force-challenge = 突增：强制挑战
admin-settings-help-security-spike-force-challenge = 在警戒模式下，对来自未知来源的匿名登录要求完成工作量证明挑战。已认证用户和已知来源不受影响。
admin-settings-key-security-spike-pace-factor-percent = 突增：登录等待(%)
admin-settings-help-security-spike-pace-factor-percent = 在警戒模式下，未知来源的登录等待时间占常规等待时间的百分比。默认 300。
admin-settings-key-security-session-binding-enabled = 会话绑定
admin-settings-help-security-session-binding-enabled = 将每个浏览器会话绑定到其客户端属性的带密钥哈希。使用持有者令牌的原生客户端、API 密钥和协议客户端不受影响。
admin-settings-key-security-session-binding-attributes = 会话绑定：属性
admin-settings-help-security-session-binding-attributes = 绑定的属性：browser、os、hints、language、country、network(IPv4 /16,IPv6 /48)。network 默认关闭，因为移动网络会变动。未知名称会被忽略。
admin-settings-key-security-session-binding-strictness = 会话绑定：严格程度
admin-settings-help-security-session-binding-strictness = 宽松模式接受浏览器更新和单个软属性变化；严格模式会标记任何软属性差异。
admin-settings-key-security-session-binding-webmail-action = 会话绑定：网页邮箱操作
admin-settings-help-security-session-binding-webmail-action = 网页邮箱会话不匹配时的处理：off（关闭）、detect（仅记录）、step-up（要求重新输入密码和第二重验证）或撤销会话。默认为 step-up。
admin-settings-key-security-session-binding-admin-action = 会话绑定：管理控制台操作
admin-settings-help-security-session-binding-admin-action = 管理控制台会话不匹配时的处理：off、detect、step-up 或撤销。默认为撤销。
admin-settings-key-security-theft-enabled = 会话盗用检测
admin-settings-help-security-theft-enabled = 检测已轮换会话 ID 的重复使用、不可能的行程，以及来自不同国家的并发使用。
admin-settings-key-security-theft-rotation-grace-secs = 盗用：轮换宽限期（秒）
admin-settings-help-security-theft-rotation-grace-secs = 会话轮换后，旧 ID 在此时长内（处理中的请求）可以再次出现，超出才算盗用。默认 30。
admin-settings-key-security-theft-min-travel-secs = 盗用：最短行程时间（秒）
admin-settings-help-security-theft-min-travel-secs = 距上一次请求不足此时长就发生国家变化，会被视为不可能的行程。国家检测可能不准确；VPN 和移动网络可能造成误报的变化。默认 3600。
admin-settings-key-security-theft-concurrent-use = 盗用：并发使用
admin-settings-help-security-theft-concurrent-use = 标记五分钟内在不同国家（或网络）交替使用的同一会话：off、country 或 network。国家检测可能不准确；VPN 和移动网络可能造成误报的变化。
admin-settings-key-security-theft-webmail-action = 盗用：网页邮箱响应
admin-settings-help-security-theft-webmail-action = 对网页邮箱会话出现不可能的行程或并发使用的响应：alert（提醒）、step-up 或撤销。重复使用已轮换的会话始终会被撤销。默认为 step-up。
admin-settings-key-security-theft-admin-action = 盗用：管理控制台响应
admin-settings-help-security-theft-admin-action = 对管理控制台会话出现不可能的行程或并发使用的响应：alert、step-up 或撤销。默认为撤销。
admin-settings-key-security-theft-retention-days = 盗用：记录保留（天）
admin-settings-help-security-theft-retention-days = 已轮换会话标记和每个会话安全记录的保留时长；请至少设为与最长会话生命周期相同。默认 15。
admin-settings-key-security-alerts-country-change = 国家变化提醒
admin-settings-help-security-alerts-country-change = 当已登录的会话在其启动时所在国家之外的国家被使用时发出警告。国家检测可能不准确；VPN 和移动网络可能造成误报的变化。除非被锁定，否则用户可以更改此项；仅当允许静音时，用户关闭它才会生效。
admin-settings-key-security-alerts-country-change-enabled = 国家变化提醒：全局开关
admin-settings-help-security-alerts-country-change-enabled = 关闭则对所有人停用国家变化提醒。国家检测可能不准确；VPN 和移动网络可能造成误报的变化。
admin-settings-key-security-alerts-country-change-cooldown-secs = 国家变化提醒：冷却时间（秒）
admin-settings-help-security-alerts-country-change-cooldown-secs = 同一会话两次国家变化提醒之间的最短间隔。每个新国家在每个会话中只提醒一次。默认 300。
admin-settings-key-security-alerts-country-change-user-mute = 国家变化提醒：允许用户静音
admin-settings-help-security-alerts-country-change-user-mute = 用户是否可以关闭自己的国家变化提醒。关闭后，即使用户自己的设置为关闭，服务器仍会持续提醒。
admin-settings-key-security-alerts-channels = 安全提醒渠道
admin-settings-help-security-alerts-channels = 会话安全警告送达用户的方式：在应用内、作为其自己邮箱中的一封邮件，或两者兼有。除非被锁定，否则用户可以更改。
admin-settings-key-security-alerts-email-enabled = 通过电子邮件发送安全提醒
admin-settings-help-security-alerts-email-enabled = 允许将会话安全警告投递到用户自己的邮箱中。不会通过 SMTP 发送邮件；没有投递会话时，警告只保留在应用和审计日志中。
admin-settings-key-security-alerts-sensitivity = 安全提醒灵敏度
admin-settings-help-security-alerts-sensitivity = “低”仅对位置发现发出警告，绝不升级验证或撤销；“正常”遵循管理员设定的操作；“高”将行程间隔加倍，并同时检查并发网络。除非被锁定，否则用户可以更改。
admin-settings-key-security-alerts-notice-retention-days = 安全通知：保留（天）
admin-settings-help-security-alerts-notice-retention-days = 应用内安全通知按账户保留的时长（每个账户最多 20 条）。默认 30。
admin-settings-inline-effective = 当前生效：
admin-settings-inline-reset = 放弃更改
admin-settings-inline-domains = 各域名的值
admin-attachments-tab-lists = 允许和拦截列表
admin-attachments-tab-limits = 限制
admin-attachments-tab-delivery = 下载和预览
admin-attachments-tab-warnings = 警告
admin-attachments-tab-test = 测试文件
admin-attachments-tab-denied = 你无权访问此标签页。
admin-attachments-group-lists = 允许和拦截列表
admin-attachments-group-limits = 大小限制
admin-attachments-group-delivery = 下载和预览
admin-attachments-group-warnings = 警告
admin-attachments-add-extension = 扩展名，例如 pdf 或 .tar.gz
admin-attachments-add-mime = MIME 类型，例如 image/png 或 image/*
admin-attachments-add = 添加
admin-attachments-remove = 移除 { $rule }
admin-attachments-filter = 筛选列表
admin-attachments-list-label = 此列表中的规则
admin-attachments-count = { $count } / { $max } 条规则
admin-attachments-empty-list = 此列表中没有规则。
admin-attachments-none-match = 没有与筛选条件匹配的规则。
admin-attachments-bulk = 粘贴多条
admin-attachments-bulk-add = 全部添加
admin-attachments-import = 导入文本文件
admin-attachments-export = 导出为文本
admin-attachments-err-extension = 不是有效的扩展名。请使用字母、数字、短横线和下划线，最多 32 个字符；各部分之间可用点分隔，如 tar.gz。
admin-attachments-err-mime = 不是有效的 MIME 类型。请使用 type/subtype，或用 type/* 表示所有子类型。
admin-attachments-err-long = 该规则太长。
admin-attachments-invalid = 未添加，不是有效的规则：{ $entries }
admin-attachments-duplicates = 已在列表中：{ $rules }
admin-attachments-overflow = 一个列表最多容纳 { $max } 条规则。其余的未添加。
admin-attachments-conflict-blockedOverAllowed = { $rule } 与另一个列表中的 { $other } 重叠。拦截规则优先，因此它会被拦截。
admin-attachments-conflict-blockedOverWarn = { $rule } 与另一个列表中的 { $other } 重叠。被拦截的文件会在任何警告之前被拒绝，因此警告永远不会显示。
admin-attachments-conflict-allowedAndWarn = { $rule } 与另一个列表中的 { $other } 重叠。该文件被允许，并会显示警告。
admin-attachments-bytes-unit = 单位
admin-attachments-unlimited = 无大小限制
admin-attachments-bytes-equals = { $size }（{ $bytes } 字节）
admin-attachments-list-allowedExtensions = 允许的扩展名
admin-attachments-list-allowedMimeTypes = 允许的 MIME 类型
admin-attachments-list-blockedExtensions = 拦截的扩展名
admin-attachments-list-blockedMimeTypes = 拦截的 MIME 类型
admin-attachments-list-warnExtensions = 警告的扩展名
admin-attachments-list-warnMimeTypes = 警告的 MIME 类型
admin-attachments-sim-title = 测试文件
admin-attachments-sim-help = 询问服务器已保存的策略会如何处理具有此名称、类型和大小的文件。不会更改或存储任何内容。绝不会读取邮件和附件。
admin-attachments-sim-filename = 文件名
admin-attachments-sim-mime = MIME 类型
admin-attachments-sim-size = 大小
admin-attachments-sim-run = 测试
admin-attachments-sim-invalid = 请输入文件名和不小于零的大小。
admin-attachments-sim-allowed = 允许
admin-attachments-sim-warn = 允许，但有警告
admin-attachments-sim-blocked = 不予投递
admin-attachments-sim-preview-yes = 允许预览。
admin-attachments-sim-preview-no = 不允许预览。
admin-attachments-sim-checked = 已检查大小：{ $size }。
admin-attachments-sim-matched = 匹配的规则位于：{ $lists }。
admin-attachments-sim-matched-none = 没有匹配的允许、拦截或警告规则。
admin-attachments-reason-downloadsDisabled = 下载已关闭。
admin-attachments-reason-attachmentTooLarge = 文件超过大小限制。
admin-attachments-reason-attachmentBlocked = 匹配到拦截规则。
admin-attachments-reason-attachmentNotAllowed = 存在允许列表，但其中没有匹配项。
admin-attachments-reason-attachmentContainerRestricted = 只要启用了任何限制，就会拒绝作为附件的邮件和 TNEF 文件，因为其内容未被列出。
admin-settings-key-mail-reply-images = 回复和转发中的图片
admin-settings-help-mail-reply-images = 回复或转发是否包含原邮件的图片：询问（默认）、始终允许、仅对可信发件人域名允许否则询问，或始终拒绝。锁定此设置可强制执行。只有在包含图片时才会读取它们。
admin-settings-key-mail-reply-image-trusted-domains = 可信的图片发件人域名
admin-settings-help-mail-reply-image-trusted-domains = 最多 200 个精确的发件人域名；当策略为可信域名时，这些域名的图片无需询问即会包含。除非此设置被锁定，否则用户可以在引用提示中添加到自己的列表。
admin-settings-key-mail-send-recall-delay-seconds = 撤销发送窗口（默认）
admin-settings-help-mail-send-recall-delay-seconds = 已发送邮件在发件箱中等待多少秒后才投递，发件人可以在此期间撤回。0 表示立即发送。默认 10，最多 1800（30 分钟）。除非你锁定此项，否则用户可以在下方限制内选择自己的值。
admin-settings-key-mail-send-recall-delay-max-seconds = 撤销发送窗口（上限）
admin-settings-help-mail-send-recall-delay-max-seconds = 用户可选的最长撤销发送窗口（秒）。服务器会将更长的请求缩短为此值。默认值和最大值均为 1800。
admin-settings-key-mail-send-allow-recall = 允许撤回已发送的邮件

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-send-allow-recall = 关闭后，邮件会立即发出，无法从发件箱撤回。定时发送不受影响；在其时间到来之前仍可取消。
admin-settings-key-mail-send-max-delayed-seconds = 定时发送时限
admin-settings-help-mail-send-max-delayed-seconds = 邮件最多可提前多久（秒）安排发送。默认 7776000（90 天），最大 31536000（1 年）。0 表示关闭定时发送。
admin-settings-key-mail-send-max-queued = 每个邮箱的排队邮件数
admin-settings-help-mail-send-max-queued = 单个邮箱可同时在发件箱中等待的邮件数（撤销窗口内或已定时）。默认 200。超出的发送会被拒绝并给出明确提示，绝不会被丢弃。
admin-attachments-sim-pick = 从文件填写（可选）
admin-attachments-mime-toggle = 显示常见 MIME 类型
admin-attachments-mime-hint = 选择常见类型，或输入任何其他 type/subtype。
admin-attachments-mime-invalid = 这不是 text/plain 这样的 type/subtype。将按输入的内容发送。
admin-attachments-mime-group-documents = 文档
admin-attachments-mime-group-images = 图片
admin-attachments-mime-group-archives = 压缩包
admin-attachments-mime-group-executables = 可执行文件和脚本
admin-attachments-mime-group-media = 音频和视频
admin-attachments-mime-group-email = 电子邮件和日历
admin-attachments-mime-group-other = 其他
admin-settings-key-cookies-enabled = Cookie 同意控件
admin-settings-help-cookies-enabled = 向访客显示 Cookie 同意控件。关闭后，应用的行为与没有该控件时相同。
admin-settings-key-cookies-mode = 同意模型
admin-settings-help-cookies-mode = optIn：访客接受之前，不存储任何可选内容。optOut：在访客拒绝之前，适用各类别的默认值。noticeOnly：仅告知访客，不提供选择。
admin-settings-key-cookies-region-mode = 严格地区的同意模型
admin-settings-help-cookies-region-mode = 用于严格地区访客的模型。same 表示在那里也使用上面的同意模型。
admin-settings-key-cookies-strict-regions = 严格地区
admin-settings-help-cookies-strict-regions = 使用严格地区同意模型的 ISO 3166-1 国家代码。默认为欧盟和欧洲经济区国家、英国和瑞士。
admin-settings-key-cookies-region-header = 国家请求头
admin-settings-help-cookies-region-header = 由代理或 CDN 设置、包含访客国家代码的请求头名称。
admin-settings-key-cookies-unknown-region = 未知国家
admin-settings-help-cookies-unknown-region = 无法确定国家时适用哪个模型：严格地区模型或常规模型。
admin-settings-key-cookies-functional-enabled = 提供功能性 Cookie
admin-settings-help-cookies-functional-enabled = 在同意控件中提供偏好设置和功能类别（主题、语言、布局记忆）。
admin-settings-key-cookies-functional-default = 功能性默认值
admin-settings-help-cookies-functional-default = 同意模型允许默认值时，功能类别的初始状态。
admin-settings-key-cookies-analytics-enabled = 提供分析 Cookie
admin-settings-help-cookies-analytics-enabled = 在同意控件中提供分析类别。
admin-settings-key-cookies-analytics-default = 分析默认值
admin-settings-help-cookies-analytics-default = 同意模型允许默认值时，分析类别的初始状态。
admin-settings-key-cookies-marketing-enabled = 提供营销 Cookie
admin-settings-help-cookies-marketing-enabled = 在同意控件中提供营销类别。
admin-settings-key-cookies-marketing-default = 营销默认值
admin-settings-help-cookies-marketing-default = 同意模型允许默认值时，营销类别的初始状态。
admin-settings-key-cookies-layout = 布局
admin-settings-help-cookies-layout = banner 是横跨页面的横幅，card 是浮动卡片，modal 是在访客做出选择前锁定页面的对话框。
admin-settings-key-cookies-position = 位置
admin-settings-help-cookies-position = 同意控件在页面上出现的位置。
admin-settings-key-cookies-reject-prominent = 拒绝与接受同样醒目
admin-settings-help-cookies-reject-prominent = 以与“全部接受”相同的大小和样式显示“全部拒绝”。
admin-settings-key-cookies-policy-url = Cookie 政策 URL
admin-settings-help-cookies-policy-url = 指向你的 Cookie 政策的链接，显示在同意控件中。
admin-settings-key-cookies-privacy-url = 隐私政策 URL
admin-settings-help-cookies-privacy-url = 指向你的隐私政策的链接，显示在同意控件中。
admin-settings-key-cookies-policy-version = 政策版本
admin-settings-help-cookies-policy-version = 当你使用 Cookie 的方式发生变化时，请提高此数字，以便再次询问访客。
admin-settings-key-cookies-lifetime-days = 同意有效期
admin-settings-help-cookies-lifetime-days = 一项决定在再次询问访客之前保持有效的天数。
admin-settings-key-cookies-reprompt-on-version = 政策版本更新时再次询问
admin-settings-help-cookies-reprompt-on-version = 当政策版本高于访客做出决定时的版本时，再次询问访客。
admin-settings-key-cookies-honour-gpc = 遵循 Global Privacy Control
admin-settings-help-cookies-honour-gpc = 将浏览器发出的 Global Privacy Control 信号视为对分析和营销的拒绝。
admin-settings-key-cookies-honour-dnt = 遵循 Do Not Track
admin-settings-help-cookies-honour-dnt = 将浏览器发出的 Do Not Track 信号视为对分析和营销的拒绝。
admin-settings-key-cookies-log-consent = 保留同意日志
admin-settings-help-cookies-log-consent = 在服务器端保留每项同意决定的记录，用于统计和你自己的存档。
admin-settings-key-cookies-log-retention-days = 同意日志保留时长
admin-settings-help-cookies-log-retention-days = 同意日志的行在被删除之前保留的天数。
admin-settings-key-cookies-scan-interval-hours = 存储扫描间隔
admin-settings-help-cookies-scan-interval-hours = 自动扫描浏览器 Cookie 和存储之间的小时数。0 表示仅按需扫描。
admin-settings-key-cookies-unclassified = 未分类的存储
admin-settings-help-cookies-unclassified = report 会单独保留无法归类的存储并列出以供审阅。其他值会在你对其分类之前将其视为该类别。
admin-settings-key-cookies-text-title = 同意控件标题
admin-settings-help-cookies-text-title = 按语言设置的同意控件标题。留空则使用内置文本。
admin-settings-key-cookies-text-body = 同意控件文本
admin-settings-help-cookies-text-body = 按语言设置的同意控件说明文本。留空则使用内置文本。
admin-settings-key-cookies-label-necessary = 必要类别的名称
admin-settings-help-cookies-label-necessary = 按语言设置的必要类别名称。留空则使用内置名称。
admin-settings-key-cookies-description-necessary = 必要类别的说明
admin-settings-help-cookies-description-necessary = 按语言设置的必要类别说明。留空则使用内置说明。
admin-settings-key-cookies-label-functional = 功能类别的名称
admin-settings-help-cookies-label-functional = 按语言设置的功能类别名称。留空则使用内置名称。
admin-settings-key-cookies-description-functional = 功能类别的说明
admin-settings-help-cookies-description-functional = 按语言设置的功能类别说明。留空则使用内置说明。
admin-settings-key-cookies-label-analytics = 分析类别的名称
admin-settings-help-cookies-label-analytics = 按语言设置的分析类别名称。留空则使用内置名称。
admin-settings-key-cookies-description-analytics = 分析类别的说明
admin-settings-help-cookies-description-analytics = 按语言设置的分析类别说明。留空则使用内置说明。
admin-settings-key-cookies-label-marketing = 营销类别的名称
admin-settings-help-cookies-label-marketing = 按语言设置的营销类别名称。留空则使用内置名称。
admin-settings-key-cookies-description-marketing = 营销类别的说明
admin-settings-help-cookies-description-marketing = 按语言设置的营销类别说明。留空则使用内置说明。
admin-settings-key-encryption-rotation-days = 数据库密钥轮换间隔（天）
admin-settings-help-encryption-rotation-days = 距上次密钥更改多少天后，将数据库密钥报告为应当轮换。0 表示不设到期日。密钥本身绝不会存储在设置中。
admin-settings-key-encryption-warnings-enabled = 报告加密发现项
admin-settings-help-encryption-warnings-enabled = 在部署评估中，将明文存储和逾期的密钥轮换显示为发现项。
admin-settings-key-retention-sweeper-enabled = 保留清理器
admin-settings-help-retention-sweeper-enabled = 运行每日清理，删除超出此页面限制的行。关闭后，所有行会一直保留，直到再次开启。
admin-settings-key-retention-sweeper-batch-rows = 每批行数
admin-settings-help-retention-sweeper-batch-rows = 一条删除语句移除的行数。批次越小，持有数据库锁的时间越短。
admin-settings-key-retention-sweeper-max-batches = 每次运行每个存储的批次数
admin-settings-help-retention-sweeper-max-batches = 一次运行中单个存储最多执行的删除语句数。旧行更多的存储会在下次运行中继续。
admin-settings-key-retention-sweeper-jitter-minutes = 启动时间分散（分钟）
admin-settings-help-retention-sweeper-jitter-minutes = 下次运行会在 24 小时之后再随机延后不超过此值的分钟数启动，避免多台服务器同时清理。
admin-settings-key-retention-sweeper-vacuum-after-rows = 回收空间阈值（行）
admin-settings-help-retention-sweeper-vacuum-after-rows = 一次运行至少删除这么多行时，会压缩数据库(VACUUM)。0 表示清理后从不压缩；每周的 vacuum 任务是独立的。
admin-settings-key-retention-password-change-audit-days = 密码更改记录（天）
admin-settings-help-retention-password-change-audit-days = 密码更改记录（账户、方式、结果）的保留天数。0 表示永久保留。
admin-settings-key-retention-sso-login-audit-days = 单点登录记录（天）
admin-settings-help-retention-sso-login-audit-days = 单点登录记录（提供方、哈希后的主体、结果）的保留天数。0 表示永久保留。
admin-settings-key-retention-dlp-audit-days = 数据防泄漏记录（天）
admin-settings-help-retention-dlp-audit-days = 数据防泄漏规则匹配记录的保留天数。0 表示永久保留。
admin-settings-key-retention-assist-audit-days = 助手访问记录（天）
admin-settings-help-retention-assist-audit-days = 助手使用某项能力的记录（执行者、能力、范围摘要）的保留天数。0 表示永久保留。
admin-settings-key-retention-alert-events-days = 已触发警报记录（天）
admin-settings-help-retention-alert-events-days = 已触发的管理员警报记录的保留天数。0 表示永久保留。
admin-settings-key-retention-job-runs-days = 后台任务历史（天）
admin-settings-help-retention-job-runs-days = 已完成的后台任务运行记录的保留天数。仍在运行的任务绝不会被移除。0 表示永久保留历史。
admin-settings-key-retention-outbox-days = 已发送和已取消的发送记录（天）
admin-settings-help-retention-outbox-days = 已投递或已取消邮件的发送记录的保留天数。正在等待、重试、失败或状态不确定的邮件绝不会被移除。0 表示永久保留。
admin-settings-key-retention-expired-credentials-grace-days = 已过期的凭据（过期后天数）
admin-settings-help-retention-expired-credentials-grace-days = 已过期的记住设备、登录挑战、登录令牌、OAuth 令牌、webhook nonce、来源失败计数器或图片链接句柄在过期后保留的天数。0 表示在下次运行时移除。
admin-settings-key-retention-ip-addresses-days = 此天数后精简已存储的 IP 地址
admin-settings-help-retention-ip-addresses-days = 经过这么多天后，身份验证历史和审计条目中的 IP 地址会被截断或哈希，并保留该行。0 表示保持地址的原始记录。
admin-settings-key-retention-ip-addresses-mode = IP 精简方式
admin-settings-help-retention-ip-addresses-mode = Truncate 保留 IPv4 的 /24 或 IPv6 的 /48 前缀。Hash 用一个简短的带密钥哈希替换地址，仍可将相同地址归为一组。
admin-settings-key-retention-user-enabled = 允许用户设置保留规则
admin-settings-help-retention-user-enabled = 用户可以为文件夹和邮件设置自己的保留规则。关闭后，现有的用户规则停止运行；下面全组织范围的规则仍会运行。
admin-settings-key-retention-user-min-days = 用户规则删除邮件前的最少天数
admin-settings-help-retention-user-min-days = 删除邮件的用户规则不能处理比此天数更新的邮件。更小的值在保存规则时会被拒绝，运行时会被提高。
admin-settings-key-retention-user-max-days = 用户规则删除邮件前最多可等待的天数
admin-settings-help-retention-user-max-days = 删除邮件的用户规则等待的时间不能超过此天数。0 表示不设上限。更大的值在保存规则时会被拒绝，运行时会被降低。
admin-settings-key-retention-user-max-rules = 每位用户的保留规则上限
admin-settings-help-retention-user-max-rules = 单个用户最多可保留的保留规则数。
admin-settings-key-retention-user-allow-permanent-delete = 用户规则可永久删除邮件
admin-settings-help-retention-user-allow-permanent-delete = 关闭后，删除规则始终将邮件移到废纸篓，并在清除之前适用撤销窗口。
admin-settings-key-retention-user-undo-days = 保留规则的撤销窗口（天）
admin-settings-help-retention-user-undo-days = 被保留规则移到废纸篓的邮件在被清除前停留的天数。用户可以选择更短的窗口。0 表示在下次运行时清除。
admin-settings-key-retention-user-mandatory = 全组织邮件规则
admin-settings-help-retention-user-mandatory = 每个账户都适用的规则，格式为 folder-role:days 条目，例如 trash:30 或 junk:30。该文件夹中超过指定天数的邮件会被清除。用户不能更改、暂停这些规则，也不能将邮件排除在外。处于法律保留下的账户的邮件会被跳过。
admin-settings-key-retention-user-protected-folders = 保留规则永不触及的文件夹
admin-settings-help-retention-user-protected-folders = 任何用户规则都不处理的文件夹，按角色（drafts、sent、inbox、archive）或文件夹名称指定。对于同一文件夹，受保护文件夹优先于全组织规则。
admin-settings-key-retention-user-templates = 向用户提供的保留模板
admin-settings-help-retention-user-templates = 用户可以在“设置”中采用的模板，按 ID 指定：newsletters30、receipts365、notifications14、trash30、junk30、read-archive180。
admin-settings-key-retention-user-run-for-restricted = 对受限账户运行用户规则
admin-settings-help-retention-user-run-for-restricted = 用户保留规则是否也对处于处理限制下的账户运行。关闭则跳过这些账户。
admin-settings-key-retention-user-interval-hours = 用户保留任务的运行间隔（小时）
admin-settings-help-retention-user-interval-hours = 单个账户的用户保留任务两次计划运行之间的小时数。
admin-settings-key-retention-user-batch-messages = 每次保留调用处理的邮件数
admin-settings-help-retention-user-batch-messages = 一次移动或删除调用处理的邮件数。最多使用 200 封。
admin-settings-key-retention-user-max-per-run = 每个账户每次运行最多更改的邮件数
admin-settings-help-retention-user-max-per-run = 用户保留任务一次运行中对单个账户最多更改的邮件数。其余的等待下次运行。
admin-settings-key-retention-user-pause-ms = 保留批次之间的暂停（毫秒）
admin-settings-help-retention-user-pause-ms = 批次之间的暂停时间（毫秒），避免大规模清理给邮件服务器造成负载。
admin-settings-key-retention-class-short-days = 短期类别上限（天）
admin-settings-help-retention-class-short-days = 运行数据的上限：会话、令牌、速率限制、任务历史和指标。在启用类别限制时适用。
admin-settings-key-retention-class-medium-days = 中期类别上限（天）
admin-settings-help-retention-class-medium-days = 登录历史、请求防护历史和安全通知的上限。不得低于短期类别；更低的值会被提高到该值。
admin-settings-key-retention-class-long-days = 长期类别保留时长（天）
admin-settings-help-retention-class-long-days = 长期类别（安全审计、管理员操作、请求、同意、墓碑记录）的保留天数，包括归档。0 表示永久保留，控制台会发出警告。不得低于中期类别。
admin-settings-key-retention-class-enforce = 让每个存储遵守其类别上限
admin-settings-help-retention-class-enforce = 开启后，每个存储自身的限制会被限定在其类别上限之内，0（保留）按上限计。关闭后，以各存储自身的限制为准。
admin-settings-key-retention-archive-enabled = 归档长期类别中已过期的行
admin-settings-help-retention-archive-enabled = 将长期类别中已过期的行移入密封的压缩归档文件，而不是删除它们。

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-retention-archive-directory = 归档目录
admin-settings-help-retention-archive-directory = 归档文件所在的目录，可以是绝对路径，也可以相对于服务器工作目录。上级目录片段(..)会被拒绝。
admin-settings-key-retention-archive-batch-rows = 每个归档文件的行数
admin-settings-help-retention-archive-batch-rows = 写入单个归档文件的行数。
admin-settings-key-retention-archive-max-segments = 每次运行每个存储的归档文件数
admin-settings-help-retention-archive-max-segments = 一次运行可为每个存储写入的归档文件数。其余的等待下次运行。
admin-settings-key-retention-archive-key-generation = 归档封存密钥代数
admin-settings-help-retention-archive-key-generation = 用于封存新归档文件的密钥代数。提高它即可轮换密钥；较早的文件仍可用其自身的代数验证。
admin-settings-key-retention-archive-verify-days = 归档检查间隔（天）
admin-settings-help-retention-archive-verify-days = 对归档文件进行完整性检查的间隔天数。
admin-settings-key-retention-archive-legal-hold = 归档的法律保留
admin-settings-help-retention-archive-legal-hold = 无论长期类别的限制如何设置，都保留每个归档文件。
admin-settings-key-logging-file-enabled = 写入日志文件
admin-settings-help-logging-file-enabled = 同时将服务器日志写入轮换文件。关闭则只输出到标准输出，这也是容器通常的用法。
admin-settings-key-logging-file-directory = 日志目录
admin-settings-help-logging-file-directory = 日志文件所在的目录，可以是绝对路径，也可以相对于服务器的工作目录。上级目录片段(..)会被拒绝。
admin-settings-key-logging-file-max-size-mb = 按大小轮换(MB)
admin-settings-help-logging-file-max-size-mb = 当前日志文件达到此大小时会被关闭，并开始写入新文件。
admin-settings-key-logging-file-rotate-hours = 按时长轮换（小时）
admin-settings-help-logging-file-rotate-hours = 当前日志文件存在这么久后会被轮换。0 表示仅按大小轮换。
admin-settings-key-logging-file-max-age-days = 轮换文件保留时长（天）
admin-settings-help-logging-file-max-age-days = 早于此时长的已轮换日志文件会被删除。
admin-settings-key-logging-file-max-files = 轮换文件数上限
admin-settings-help-logging-file-max-files = 超出此数量的最旧已轮换文件会被删除。
admin-settings-key-logging-file-max-total-mb = 总大小上限(MB)
admin-settings-help-logging-file-max-total-mb = 会删除最旧的已轮换文件，直到日志目录小于此大小。
admin-settings-key-logging-file-compress = 压缩轮换文件
admin-settings-help-logging-file-compress = 已轮换的日志文件使用 gzip 压缩。
admin-settings-key-logging-level = 日志级别
admin-settings-help-logging-level = 适用于下方没有自身级别的组件的级别。在读取此设置之前，启动时由 MW_LOG 环境变量设定级别。
admin-settings-key-logging-component-levels = 组件日志级别
admin-settings-help-logging-component-levels = 每行一个 component=level 条目，例如 mw_imap=debug。级别有 error、warn、info、debug 和 trace。
admin-settings-key-logging-redaction-addresses = 日志中的电子邮件地址
admin-settings-help-logging-redaction-addresses = Full 保持地址原样记录。Truncated 保留首字母和域名。Hashed 将本地部分替换为简短哈希。None 移除该地址。
admin-settings-key-logging-redaction-ips = 日志中的 IP 地址
admin-settings-help-logging-redaction-ips = Full 保持地址原样记录。Truncated 保留 IPv4 的 /24 或 IPv6 的 /48 前缀。Hashed 将地址替换为简短哈希。None 将其移除。
admin-settings-key-logging-redaction-tokens = 日志中的机密信息
admin-settings-help-logging-redaction-tokens = 从日志行中剔除持有者令牌、API 密钥、URL 中的密码和类似机密信息。
admin-settings-key-retention-sessions-idle-days = 未使用的会话（天）
admin-settings-help-retention-sessions-idle-days = 带有地址和 User-Agent 的浏览器或原生客户端会话记录在被删除前可闲置的天数。会话超时优先生效；此项仅清除遗留的行。0 表示保留。
admin-settings-key-policy-public-pages = 公开政策页面
admin-settings-help-policy-public-pages = 无需登录即可在 /privacy 提供生成的隐私政策，在 /cookies-policy 提供 Cookie 政策。
admin-settings-key-policy-auto-version = 自动政策版本
admin-settings-help-policy-auto-version = 当政策所描述的某项设置发生变化时，记录新的政策版本。关闭：更改等待“发布”。
admin-settings-key-policy-reconsent-on-change = 新政策版本时再次请求 Cookie 同意
admin-settings-help-policy-reconsent-on-change = 记录新政策版本时提高 Cookie 政策版本，使访客被再次询问（需要 cookies.repromptOnVersion）。
admin-settings-key-policy-controller-name = 控制者名称
admin-settings-help-policy-controller-name = 决定数据处理目的和方式的组织名称，显示在隐私政策中。
admin-settings-key-policy-controller-address = 控制者地址
admin-settings-help-policy-controller-address = 控制者的邮寄地址，显示在隐私政策中。
admin-settings-key-policy-legal-bases = 法律依据
admin-settings-help-policy-legal-bases = 你所依据的法律依据，用你自己的话表述，显示在隐私政策中。
admin-settings-key-policy-hosting-provider = 托管提供方
admin-settings-help-policy-hosting-provider = 谁托管数据以及托管在何处，显示在隐私政策中。
admin-settings-key-policy-hosting-transfers = 向托管国家以外的传输
admin-settings-help-policy-hosting-transfers = 向其他国家传输数据的任何情况以及你采用的保障措施，显示在隐私政策中。
admin-settings-key-policy-supervisory-authority = 监管机构
admin-settings-help-policy-supervisory-authority = 用户可以向其投诉的机构，显示在隐私政策中。
admin-settings-key-dpia-suggest = 建议开展影响评估
admin-settings-help-dpia-suggest = 当启用了助手、打开跟踪或分析而尚无评估时，列出建议开展的影响评估。
admin-settings-key-dpia-review-months = 影响评估复审间隔
admin-settings-help-dpia-review-months = 评估获批时设定的复审日期距今的月数。
admin-settings-key-dpia-reminder-days = 影响评估提醒窗口
admin-settings-help-dpia-reminder-days = 在复审日期前多少天，将评估列为到期。
admin-settings-key-restriction-enabled = 处理限制
admin-settings-help-restriction-enabled = 管理员或已批准的请求是否可以限制对某人数据的处理。
admin-settings-key-restriction-auto-apply-on-approval = 请求获批时自动限制
admin-settings-help-restriction-auto-apply-on-approval = 限制请求一经批准就立即应用限制。关闭：由管理员在“合规 > 限制”中应用。
admin-settings-key-restriction-mail-delivery = 限制期间的新邮件
admin-settings-help-restriction-mail-delivery = 管理员为限制生效期间到达的邮件所选的处理方式：保留、排队或拒收。此服务器不接收邮件；该值会被记录和显示，由邮件服务器据此执行。
admin-settings-key-restriction-default-expiry-days = 默认限制时长（天）
admin-settings-help-restriction-default-expiry-days = 新限制自行结束之前的天数。0 表示保持到有人解除为止。
admin-settings-key-restriction-notify-user = 通知当事人
admin-settings-help-restriction-notify-user = 在应用或解除限制时向当事人发送通知。
admin-settings-key-restriction-scope-indexing = 暂停搜索索引和嵌入
admin-settings-help-restriction-scope-indexing = 新限制中是否默认暂停搜索索引和嵌入。
admin-settings-key-restriction-scope-assistant = 暂停助手
admin-settings-help-restriction-scope-assistant = 新限制中是否默认暂停助手。
admin-settings-key-restriction-scope-analytics = 暂停分析和打开跟踪
admin-settings-help-restriction-scope-analytics = 新限制中是否默认暂停分析和打开跟踪。
admin-settings-key-restriction-scope-classification = 暂停规则和自动归档
admin-settings-help-restriction-scope-classification = 新限制中是否默认暂停规则和自动归档。
admin-settings-key-restriction-scope-push = 暂停推送通知
admin-settings-help-restriction-scope-push = 新限制中是否默认暂停推送通知。
admin-settings-key-restriction-scope-third-party = 暂停已连接的服务和插件
admin-settings-help-restriction-scope-third-party = 新限制中是否默认暂停已连接的服务和插件。
admin-settings-key-consent-enabled = 目的同意
admin-settings-help-consent-enabled = 向用户显示各项目的和同意选择，并在运行时强制执行。
admin-settings-key-consent-receipts-retention-days = 同意回执保留时长（天）
admin-settings-help-consent-receipts-retention-days = 同意回执的保留天数。0 表示保留到该人员被删除为止。
admin-settings-key-consent-art22-notice = 自动化处理通知
admin-settings-help-consent-art22-notice = 向用户显示关于助手和搜索功能的通俗易懂的通知，以及各项功能的开关。
admin-settings-key-consent-analytics-basis = 法律依据：使用情况统计
admin-settings-help-consent-analytics-basis = 向用户显示的使用情况统计的法律依据。只有“同意”会让该功能等待用户的确认；其他依据则在当事人撤回之前适用。由管理员决定哪种依据合适。
admin-settings-key-consent-analytics-version = 文本版本：使用情况统计
admin-settings-help-consent-analytics-version = 当措辞或处理方式发生变化时，请提高此数字；同意过旧版本的人会被再次询问。
admin-settings-key-consent-analytics-text = 自定义措辞：使用情况统计
admin-settings-help-consent-analytics-text = 按语言为此目的自定义的文本。留空则使用内置措辞。
admin-settings-key-consent-analytics-tag-basis = 法律依据：分析提供方脚本
admin-settings-help-consent-analytics-tag-basis = 向用户显示的分析提供方脚本的法律依据。只有“同意”会让该功能等待用户的确认；其他依据则在当事人撤回之前适用。由管理员决定哪种依据合适。
admin-settings-key-consent-analytics-tag-version = 文本版本：分析提供方脚本
admin-settings-help-consent-analytics-tag-version = 当措辞或处理方式发生变化时，请提高此数字；同意过旧版本的人会被再次询问。
admin-settings-key-consent-analytics-tag-text = 自定义措辞：分析提供方脚本
admin-settings-help-consent-analytics-tag-text = 按语言为此目的自定义的文本。留空则使用内置措辞。
admin-settings-key-consent-assistant-basis = 法律依据：助手
admin-settings-help-consent-assistant-basis = 向用户显示的助手的法律依据。只有“同意”会让该功能等待用户的确认；其他依据则在当事人撤回之前适用。由管理员决定哪种依据合适。
admin-settings-key-consent-assistant-version = 文本版本：助手
admin-settings-help-consent-assistant-version = 当措辞或处理方式发生变化时，请提高此数字；同意过旧版本的人会被再次询问。
admin-settings-key-consent-assistant-text = 自定义措辞：助手
admin-settings-help-consent-assistant-text = 按语言为此目的自定义的文本。留空则使用内置措辞。
admin-settings-key-consent-push-basis = 法律依据：推送通知
admin-settings-help-consent-push-basis = 向用户显示的推送通知的法律依据。只有“同意”会让该功能等待用户的确认；其他依据则在当事人撤回之前适用。由管理员决定哪种依据合适。
admin-settings-key-consent-push-version = 文本版本：推送通知
admin-settings-help-consent-push-version = 当措辞或处理方式发生变化时，请提高此数字；同意过旧版本的人会被再次询问。
admin-settings-key-consent-push-text = 自定义措辞：推送通知
admin-settings-help-consent-push-text = 按语言为此目的自定义的文本。留空则使用内置措辞。
admin-settings-key-consent-open-tracking-basis = 法律依据：打开跟踪
admin-settings-help-consent-open-tracking-basis = 向用户显示的打开跟踪的法律依据。只有“同意”会让该功能等待用户的确认；其他依据则在当事人撤回之前适用。由管理员决定哪种依据合适。
admin-settings-key-consent-open-tracking-version = 文本版本：打开跟踪
admin-settings-help-consent-open-tracking-version = 当措辞或处理方式发生变化时，请提高此数字；同意过旧版本的人会被再次询问。
admin-settings-key-consent-open-tracking-text = 自定义措辞：打开跟踪
admin-settings-help-consent-open-tracking-text = 按语言为此目的自定义的文本。留空则使用内置措辞。
admin-settings-key-consent-remote-content-basis = 法律依据：远程图片
admin-settings-help-consent-remote-content-basis = 向用户显示的远程图片的法律依据。只有“同意”会让该功能等待用户的确认；其他依据则在当事人撤回之前适用。由管理员决定哪种依据合适。
admin-settings-key-consent-remote-content-version = 文本版本：远程图片

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-consent-remote-content-version = 当措辞或处理方式发生变化时，请提高此数字；同意过旧版本的人会被再次询问。
admin-settings-key-consent-remote-content-text = 自定义措辞：远程图片
admin-settings-help-consent-remote-content-text = 按语言为此目的自定义的文本。留空则使用内置措辞。
admin-settings-key-consent-integrations-basis = 法律依据：已连接的服务
admin-settings-help-consent-integrations-basis = 向用户显示的已连接服务的法律依据。只有“同意”会让该功能等待用户的确认；其他依据则在当事人撤回之前适用。由管理员决定哪种依据合适。
admin-settings-key-consent-integrations-version = 文本版本：已连接的服务
admin-settings-help-consent-integrations-version = 当措辞或处理方式发生变化时，请提高此数字；同意过旧版本的人会被再次询问。
admin-settings-key-consent-integrations-text = 自定义措辞：已连接的服务
admin-settings-help-consent-integrations-text = 按语言为此目的自定义的文本。留空则使用内置措辞。
admin-settings-key-consent-telemetry-basis = 法律依据：加载时间测量
admin-settings-help-consent-telemetry-basis = 向用户显示的加载时间测量的法律依据。只有“同意”会让该功能等待用户的确认；其他依据则在当事人撤回之前适用。由管理员决定哪种依据合适。
admin-settings-key-consent-telemetry-version = 文本版本：加载时间测量
admin-settings-help-consent-telemetry-version = 当措辞或处理方式发生变化时，请提高此数字；同意过旧版本的人会被再次询问。
admin-settings-key-consent-telemetry-text = 自定义措辞：加载时间测量
admin-settings-help-consent-telemetry-text = 按语言为此目的自定义的文本。留空则使用内置措辞。
admin-settings-key-breach-enabled = 数据泄露登记册
admin-settings-help-breach-enabled = 启用登记册和通知计时。
admin-settings-key-breach-notice-hours = 通知监管机构的时限（小时）
admin-settings-help-breach-notice-hours = 从获知到通知监管机构的小时数。通常为 72。
admin-settings-key-breach-reminders-hours = 提醒时间
admin-settings-help-breach-reminders-hours = 在截止时间前多少小时发出提醒，例如 6、24 和 48。
admin-settings-key-breach-reminders-overdue-repeat-hours = 逾期重复提醒
admin-settings-help-breach-reminders-overdue-repeat-hours = 截止时间已过后，两次提醒之间的小时数。0 表示只提醒一次。
admin-settings-key-breach-authority-name = 监管机构
admin-settings-help-breach-authority-name = 印在监管机构通知上的名称。
admin-settings-key-breach-authority-url = 监管机构通报页面
admin-settings-help-breach-authority-url = 监管机构通报页面的地址。
admin-settings-key-breach-authority-contact = 监管机构联系方式
admin-settings-help-breach-authority-contact = 监管机构的联系方式。
admin-settings-key-breach-dpo-name = 数据保护联系人姓名
admin-settings-help-breach-dpo-name = 在通知中作为联系人给出的姓名。
admin-settings-key-breach-dpo-email = 数据保护联系人邮箱
admin-settings-help-breach-dpo-email = 在通知中作为联系人给出的地址。
admin-settings-key-breach-dpo-phone = 数据保护联系人电话
admin-settings-help-breach-dpo-phone = 在通知中给出的电话号码。
admin-settings-key-breach-template-subject = 邮件主题
admin-settings-help-breach-template-subject = 按语言设置的发给受影响人员的邮件主题。
admin-settings-key-breach-template-body = 邮件正文
admin-settings-help-breach-template-body = 按语言设置的发给受影响人员的邮件正文。
admin-settings-key-breach-notify-audit = 在审计日志中记录通知
admin-settings-help-breach-notify-audit = 提醒和检测结果会成为安全事件。
admin-settings-key-breach-notify-webhook-urls = Webhook 地址
admin-settings-help-breach-notify-webhook-urls = 接收提醒和邮件通知的地址。请将其视为机密；每次调用都会签名。
admin-settings-key-breach-notify-operator-addresses = 运维人员邮箱
admin-settings-help-breach-notify-operator-addresses = 在自己的收件箱中接收提醒的本地邮箱地址。
admin-settings-key-breach-subjects-in-app = 当事人账户内的通知
admin-settings-help-breach-subjects-in-app = 在当事人的账户内显示该消息。
admin-settings-key-breach-subjects-inbox = 发给当事人邮箱的邮件
admin-settings-help-breach-subjects-inbox = 将邮件投递到当事人自己的邮箱。
admin-settings-key-breach-subjects-webhook = 同时发送到 webhook
admin-settings-help-breach-subjects-webhook = 也将邮件文本发送到 webhook 地址。
admin-settings-key-breach-retention-years = 登记册保留年数
admin-settings-help-breach-retention-years = 已关闭的事件在登记册中保留的年数。
admin-settings-key-breach-auto-enabled = 自动化
admin-settings-help-breach-auto-enabled = 根据检测结果创建事件草稿。
admin-settings-key-breach-auto-start-clock = 草稿计时
admin-settings-help-breach-auto-start-clock = 在草稿创建时开始计时，或仅在确认后才开始。
admin-settings-key-breach-auto-assignee = 草稿分配给
admin-settings-help-breach-auto-assignee = 草稿分配给谁。
admin-settings-key-breach-auto-escalate-after-hours = 超过多少小时后升级
admin-settings-help-breach-auto-escalate-after-hours = 草稿在被升级前等待的小时数。0 表示从不升级。
admin-settings-key-breach-auto-escalate-to = 升级给
admin-settings-help-breach-auto-escalate-to = 被升级的草稿分配给谁。
admin-settings-key-breach-auto-window-minutes = 统计窗口（分钟）
admin-settings-help-breach-auto-window-minutes = 规则统计事件的分钟数。
admin-settings-key-breach-auto-theft-threshold = 会话盗用提醒
admin-settings-help-breach-auto-theft-threshold = 窗口内达到多少次提醒就创建草稿。0 表示关闭。
admin-settings-key-breach-auto-credential-stuffing-threshold = 登录喷洒或突增
admin-settings-help-breach-auto-credential-stuffing-threshold = 窗口内达到多少次事件就创建草稿。0 表示关闭。
admin-settings-key-breach-auto-mass-forward-threshold = 批量转发规则
admin-settings-help-breach-auto-mass-forward-threshold = 窗口内达到多少次事件就创建草稿。0 表示关闭。
admin-settings-key-breach-auto-role-escalation-threshold = 管理员角色变更
admin-settings-help-breach-auto-role-escalation-threshold = 窗口内达到多少次变更就创建草稿。0 表示关闭。
admin-settings-key-breach-auto-export-volume-threshold = 异常导出
admin-settings-help-breach-auto-export-volume-threshold = 窗口内达到多少次导出就创建草稿。0 表示关闭。
admin-settings-key-breach-auto-audit-chain-threshold = 审计链失败
admin-settings-help-breach-auto-audit-chain-threshold = 达到多少次失败就创建草稿。0 表示关闭。
admin-settings-key-breach-auto-backup-anomaly-threshold = 备份或恢复失败
admin-settings-help-breach-auto-backup-anomaly-threshold = 窗口内达到多少次失败就创建草稿。0 表示关闭。
admin-settings-key-breach-auto-dsr-export-threshold = 大量数据请求导出
admin-settings-help-breach-auto-dsr-export-threshold = 窗口内达到多少次导出就创建草稿。0 表示关闭。
admin-settings-key-ratelimit-queue-ms = 等待空闲位置的最长时间
admin-settings-help-ratelimit-queue-ms = 请求等待空闲位置（JMAP 并发、图片解码器、预览工作进程）的最长时间，超时后以 429 和 Retry-After 拒绝。范围 0-30000 毫秒；默认 3000。0 表示立即拒绝。
admin-settings-key-ratelimit-jmap-per-account-concurrent = 每个账户同时进行的 JMAP 请求数
admin-settings-help-ratelimit-jmap-per-account-concurrent = 一个已登录账户可同时运行的 JMAP 请求数，涵盖其所有标签页和设备。不会与其他账户或同一地址背后的其他用户共享。范围 4-64；默认 8。
admin-settings-key-ratelimit-jmap-global-concurrent = 所有账户同时进行的 JMAP 请求数
admin-settings-help-ratelimit-jmap-global-concurrent = 整个服务器进程同时运行的 JMAP 请求数，所有账户合计。超出的请求会等待排队时间。范围 4-4096；默认 256。
admin-settings-key-ratelimit-session-burst = 每个会话连续的计时报告数
admin-settings-help-ratelimit-session-burst = 一个会话可连续发送的客户端计时报告数。该端点按会话而不是按地址计数。范围 1-1000；默认 60。
admin-settings-key-ratelimit-session-per-minute = 每分钟恢复的计时报告数
admin-settings-help-ratelimit-session-per-minute = 会话在用完突发额度后，每分钟恢复的客户端计时报告数。范围 1-600；默认 30。
admin-settings-key-ratelimit-anon-burst = 连续的匿名同意请求数
admin-settings-help-ratelimit-anon-burst = 一个地址可连续向匿名 Cookie 同意路由发出的请求数。按位于同一地址之后的办公室来确定大小。范围 1-1000；默认 60。
admin-settings-key-ratelimit-anon-per-minute = 每分钟恢复的匿名同意请求数
admin-settings-help-ratelimit-anon-per-minute = 一个地址在匿名 Cookie 同意路由上每分钟恢复的请求数。范围 1-600；默认 30。
admin-settings-key-ratelimit-trusted-multiplier = 已知来源的账户额度(%)
admin-settings-help-ratelimit-trusted-multiplier = 当请求来自有成功登录记录的来源时，已登录邮箱可获得的账户请求额度的百分比。100 保持基础额度。它绝不会放宽匿名、管理员或 User-Agent 预算，也不会放宽正在生效的拦截。范围 100-1000；默认 150。

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-attachments-bar-click-to-toggle = 单击阅读器附件栏上的任意位置是否会展开或折叠它。关闭时，只有箭头按钮起作用。

admin-settings-help-attachments-bar-download-all = 附件栏是否提供“全部下载”按钮，将允许的附件保存为一个 ZIP 文件。

admin-settings-help-attachments-bar-show-download-buttons = 当栏中的每个附件显示其下载按钮时：悬停或聚焦（默认）、始终或从不。下载仍遵循附件策略和 mail.allowDownload。

admin-settings-help-attachments-bar-zip-inline-images = 消息文本中显示的图像是否包含在全部下载ZIP中。默认关闭。

admin-settings-help-cap-reading-delete-sender = 允许用户启用阅读器操作，在确认后将所有匹配的发件人邮件移至“废纸篓”。默认禁用。功能配置文件也必须允许这样做。

admin-settings-help-compose-feature-family = 通过功能 ID 打开或关闭 Composer 控件。默认开启；锁定它以阻止用户更改它。

admin-settings-help-compose-headers-allow-auto-submitted = 允许用户在消息上设置自动提交 (RFC 3834)。默认关闭：告诉其他系统不要回复。

admin-settings-help-compose-headers-allow-custom = 允许用户添加自己的标头（最多 20 个，检查名称，拒绝受保护的标头）。

admin-settings-help-compose-headers-allow-return-receipt-to = 允许用户使用旧的 Return-Receipt-To 标头请求收据。默认关闭。

admin-settings-help-compose-headers-allow-threading-edit = 允许用户编辑回复和参考，或从回复开始新线程。

admin-settings-help-compose-headers-forced = 每行一个名称=值对，添加到从域发送的每条消息中并替换用户的值。受保护的标头被拒绝。

admin-settings-help-compose-insert-image-resize-default = “来自文件的图片”对话框的大小开头为：原始、大 (1600 像素) 或中 (1024 像素)。

admin-settings-help-compose-insert-max-inline-image-mb = 大于此的图片不放置在消息正文中；它们仍然可以附着。 1 到 50。

admin-settings-help-compose-layout-default-font = 新消息开头的字体系列、大小和颜色，如 family|size|#colour。空使用主题字体。

admin-settings-help-compose-reply-to-allow-override = 关闭时，身份的回复按原样使用，用户无法编辑或删除它。

admin-settings-help-compose-reply-to-show = 撰写者是否显示回复行：仅当身份或原始消息有一个、始终或从不时。

admin-settings-help-compose-sent-folder-fixed-id = 当规则为“固定文件夹”时使用的文件夹角色或 ID。

admin-settings-help-compose-sent-folder-mode = 已发送邮件的副本所在位置：普通的“已发送”文件夹、一个固定文件夹、按收件人域、身份或主题规则。

admin-settings-help-compose-sent-folder-per-identity = 每行一个标识=文件夹对。当规则是“按身份”时使用。

admin-settings-help-compose-sent-folder-rules = 每行一条规则，domain:example.com=文件夹或subject:text=文件夹。第一场比赛获胜。

admin-settings-help-compose-sent-folder-skip-copy = 切勿保留已发送消息的副本。管理员可以强制打开或关闭此功能。

admin-settings-help-compose-styles-custom = 用户自己的作曲家风格为JSON，由风格库编写。用户设置；这里没有什么可配置的。

admin-settings-help-compose-styles-default-set = 作曲家风格库的外观：默认、现代、经典或紧凑。

admin-settings-help-compose-styles-max-custom = 用户可以在编辑器中保留大多数自定义样式。 0 关闭创建样式。

admin-settings-help-contacts-save-from-message-add-note = 将注释“从<日期>的消息中保存”添加到从消息中保存的联系人中。可以锁定用户和域覆盖。

admin-settings-help-contacts-save-from-message-default-book = 接收消息中保存的联系人的通讯录id； “默认”使用默认地址簿。可以锁定用户和域覆盖。

admin-settings-help-contacts-save-from-message-default-scope = 在“保存联系人”对话框中首先勾选哪些参与者：仅限发件人、发件人和回复者或所有人。可以锁定用户和域覆盖。

admin-settings-help-contacts-save-from-message-enabled = 提供在消息菜单中保存此消息中的联系人。关闭时该项目被隐藏。可以锁定用户和域覆盖。

admin-settings-help-contacts-save-from-message-skip-automated = 除非手动勾选，否则请将无回复地址、通知地址和退回邮件地址保留在“保存联系人”对话框之外。可以锁定用户和域覆盖。

admin-settings-help-contacts-save-sender-enabled = 提供一键将发件人保存为消息菜单中的联系人项目的功能。关闭时该项目被隐藏。可以锁定用户和域覆盖。

admin-settings-help-contacts-save-sender-use-reply-to = 当发件人是邮件列表或通知地址时，保存回复地址而不是发件人地址。可以锁定用户和域覆盖。

admin-settings-help-dsr-admin-step-up = 开启（默认）：批准数据请求、运行删除或批量批准的管理员必须首先再次确认是他们本人（密码和第二因素或密钥）。确认持续五分钟。 Off 删除额外的检查；该决定仍记录在请求跟踪中。

admin-settings-help-export-default-format = 导出对话框开始的格式。

admin-settings-help-export-default-scope = 导出对话框开始的范围：消息、对话、选择或文件夹。

admin-settings-help-export-dialog-enabled = 在消息视图和消息菜单中导出会打开一个对话框。关闭时，这些菜单会将一条消息下载为 .eml。

admin-settings-help-export-folder-enabled = 允许用户将整个文件夹下载为ZIP 消息。每次出口都经过审核。

admin-settings-help-export-folder-format = 文件夹导出提供的格式：.eml 文件的 ZIP、包含 mbox 文件的 ZIP，或两者兼而有之。

admin-settings-help-export-folder-include-subfolders = 文件夹导出是否默认包含其子文件夹，以文件夹结构形式保存在ZIP中。

admin-settings-help-export-folder-max-bytes = 一个文件夹导出可以容纳的最大字节数的消息。拒绝使用所示大小的较大文件夹。

admin-settings-help-export-folder-max-messages = 一个文件夹导出可容纳的最多邮件。

admin-settings-help-export-formats = 导出对话框提供的格式：eml、mbox、txt、md、msg、docx、csv、json、pdf。服务器拒绝列出的服务器构建格式。

admin-settings-help-export-message-download-enabled = 允许用户将邮件下载为 .eml（仅包含标题）或 mbox 文件。

admin-settings-help-folders-hide-empty = 隐藏侧边栏中的空文件夹。用户可以更改默认值或覆盖单个文件夹的默认值，除非此设置被锁定。没有邮件被删除。

admin-settings-help-folders-pinned-hide-from-tree = 固定文件夹显示在文件夹树上方的固定组中，并保留其在树中的位置。打开会隐藏树中没有子文件夹的固定文件夹，因此它仅显示在固定组中。带有子文件夹的固定文件夹保留在树中。

admin-settings-help-mail-badge-auth = 当读卡器标题显示DMARC身份验证徽章时：始终、仅在值得注意时或从不。

admin-settings-help-mail-badge-density = 读卡器标头中的徽章尺寸：紧凑或舒适。

admin-settings-help-mail-badge-folder = 当阅读器标题显示文件夹徽章时：始终、仅在值得注意时或从不。

admin-settings-help-mail-badge-headers = 在阅读器中显示消息的“过期”、“回复者”、“敏感度”和“取代”标头的徽章。

admin-settings-help-mail-badge-labels = 读者标题徽章是否显示带文本的图标，或仅显示图标。

admin-settings-help-mail-badge-list = 当阅读器标题显示邮件列表徽章时：始终、仅在值得注意时或从不。它需要 List-Id 或 List-Unsubscribe 标头。

admin-settings-help-mail-badge-sanitized = 当读卡器标题显示已清理的HTML徽章时：始终、仅在值得注意时或从不。

admin-settings-help-mail-badge-spam = 当阅读器标题显示垃圾邮件分数徽章时：始终、仅在值得注意时或从不。

admin-settings-help-mail-brand-avatars-require-auth = 从不：只要域名匹配就显示徽标。 known-failure（默认）：显示它，除非消息的缓存DKIM或DMARC结果失败；未经检查的消息保留徽标。始终：仅在品牌自己的域名的 DMARC 与 DKIM 对齐后显示。可以锁定用户和域覆盖。

admin-settings-help-mail-compose-header-auto-collapse-seconds = 在完整的收件人和主题标题折叠到其摘要之前，正文必须保持焦点几秒钟。选择 0 禁用定时折叠，或选择 1-60 秒。用户可以选择，除非它被锁定。

admin-settings-help-mail-compose-priority-default = 新消息在编辑器中开始的优先级。

admin-settings-help-mail-compose-priority-enabled = 在作曲家选项组中显示“正常”、“高”和“低”优先级选择器。关闭时，将发送不带优先级标头的新邮件。

admin-settings-help-mail-expiry-hide-expired = 如何处理过期日期已过的邮件：什么都没有、将其显示为灰色或建议将其存档。

admin-settings-help-mail-list-keep = 始终保留发件人和邮件列表，每行一个作为sender:address或list:List-Id。清理此发件人、取消订阅建议并保留规则跳过它们。

admin-settings-help-mail-list-page-size = 每次用户滚动到末尾时列表会获取多少条消息。默认 50，从 20 到 200。用户选择自己的值，除非您锁定此值。

admin-settings-help-mail-list-unsubscribe-mode = 当列表提供一键式请求时，取消订阅的工作原理：先询问、立即发送，或者仅打开列表的链接或草稿。

admin-settings-help-mail-message-list-group = 按接收或发送日期、发件人、主题、重要性或大小对对话进行分组。除非锁定，否则用户可以覆盖此设置。

admin-settings-help-mail-priority-marks = 哪个优先级标记消息列表、对话和阅读器显示：高和低、仅高或无。用户在“设置”>“文件夹”和侧边栏下进行选择，除非锁定。

admin-settings-help-mail-priority-marks-style = 优先级标记的外观：图标、带有“高”或“低”字样的图标或细条。图标和文字保持意义，没有颜色。

admin-settings-help-mail-priority-send-headers = 高优先级或低优先级邮件在发送时获得哪些标头：Importance、X-Priority 和 X-MSMail-Priority 中的任何一个。

admin-settings-help-mail-priority-trust-sender-headers = 是否显示发送者设置的优先级：始终，仅当消息通过对齐的 DMARC 检查时，或从不。对将每封邮件标记为高优先级的发件人使用“可信”或“从不”。

admin-settings-help-mail-reply-from-header-priority = 消息头按此顺序读取，以查找在选择回复身份时消息发送到的地址。名称：递送至、X 原始至、信封至、X 转发至、重新发送至、收件人、抄送。未读取的标头不会被读取。

admin-settings-help-mail-search-default-scope = 在当前文件夹或所有文件夹中开始新的普通邮件搜索。这控制初始默认值，包括锁定时；用户仍然可以为特定搜索选择不同的范围。已保存的查询和显式范围的查询将被保留。

admin-settings-help-metrics-dimensions-enabled = 存储按客户端类型（Web）和设备类别（桌面、移动）划分的已保存统计信息。关闭以存储每个指标和分钟的组合行。

admin-settings-help-metrics-persist-enabled = 每个刷新间隔以及服务器停止时将性能统计数据（图像代理速度、浏览器加载时间、速率限制和请求保护计数）保存到数据库中，以便图表在重新启动后保留其历史记录。没有任何东西可以识别个人、地址或消息。关闭以仅将它们保留在内存中。

admin-settings-help-metrics-persist-flush-seconds = 性能统计信息写入数据库的频率（5 到 3600 秒）。没有任何警告就被杀死的服务器最多损失这么多。

admin-settings-help-observability-client-capture-console = 还报告 console.error 调用，每页最多五次。默认情况下关闭，因为库会记录预期条件。

admin-settings-help-observability-client-capture-csp = 报告浏览器检测到的内容安全策略违规行为，仅包含被阻止的指令和来源。默认开启。

admin-settings-help-observability-client-enabled = 从 Web 应用程序收集错误报告（错误边界、未捕获的错误、失败的块、工作错误、CSP 违规）以及已清理的堆栈跟踪，显示在可观察性 > 客户端错误下。默认开启。报告不包含消息内容、地址、令牌、附件名称或 URL 查询。关闭停止采集；现有报告将随着保留而过期。

admin-settings-help-observability-client-forward = 还将每个已清理的报告发送到使用 MW_ERROR_FORWARD_URL 配置的错误接收器。默认关闭。如果没有配置接收器，则不会发送任何内容。

admin-settings-help-observability-client-include-account-hash = 将帐户的密钥单向哈希添加到从登录会话发送的报告中，以便无需命名即可区分来自一个帐户的报告。默认关闭。当隐私密钥轮换时，哈希值会发生变化。

admin-settings-help-observability-client-max-per-session = 一页加载可能会发送大多数错误报告（1 到 1000）。 60 秒内重复出现相同错误算一次。默认 20。

admin-settings-help-observability-client-sample-rate = 浏览器发送的错误报告的比例，从 0 到 1。默认为 1（全部）。每个页面加载仍然停止在每个会话限制。

admin-settings-help-observability-client-source-maps = 当构建发布隐藏源映射（使用 MW_SOURCEMAP=hidden 构建）时，将堆栈跟踪中的包位置解析为源文件。地图只能由服务器读取，并且不会提供给浏览器。默认开启；如果没有地图，则会显示原始帧。

admin-settings-help-privacy-error-reports = 当应用程序的一部分出现故障时，浏览器是否发送错误报告。默认开启。全局、域和用户范围：此处设置默认或锁定；人们可以在“设置”>“隐私”中自行将其关闭，除非它被锁定。

admin-settings-help-privacy-image-cache-max-stale-secs = 过了几秒后，缓存的图像就会被保留，以便在无法到达原点时做出响应。切勿低于重新验证窗口。范围 0–604800；默认86400。

admin-settings-help-privacy-image-cache-stale-secs = 缓存的图像在其新鲜度过后几秒仍会立即提供，同时在后台重新验证。零将其关闭。范围 0–86400；默认 3600。

admin-settings-help-privacy-image-resize-enabled = 将超大远程图像缩小到读者要求的宽度（320、640、960、1280 或 1600 像素）。动画图像未缩放。默认关闭。

admin-settings-help-privacy-image-resize-format = 自动选择构建中最好的编码器，即 PNG。默认自动。

admin-settings-help-privacy-image-resize-max-pixels = 解码像素数多于此的图像未缩放；在任何解码之前从标头读取大小。范围 100000–64000000；默认 4000000。

admin-settings-help-privacy-image-resize-max-width = 调整大小后生成的最宽图像。范围 320–1600；默认 1600。

admin-settings-help-privacy-sessions-show-linked-origin = 设置 > 安全 > 会话如何命名链接邮箱访问会话来源的帐户：完整地址、屏蔽名称第一部分的地址或什么都没有。该值仅向会话所有者显示，而不会向其他用户显示。默认值：完整地址。

admin-settings-help-properties-folder-enabled = 在文件夹菜单和快捷方式上提供属性。关闭时，该对话框不可用。

admin-settings-help-properties-folder-notes-enabled = 允许对每个文件夹进行描述或注释。

admin-settings-help-properties-folder-notes-max-length = 文件夹便笺可以容纳的最多字符。

admin-settings-help-properties-folder-overrides-allowed = 用户可以为单个文件夹覆盖哪些设置。未列出的覆盖将被隐藏，并且任何存储的值都会被忽略。

admin-settings-help-properties-folder-sections = 文件夹属性对话框的各个部分：常规、远程、大小、脱机、覆盖、注释、操作。未列出的部分被隐藏。

admin-settings-help-properties-folder-size-compute = 当计算出服务器上文件夹的大小时：从不、当对话框打开时或在后台。对大邮箱进行计数会导致服务器扫描成本。

admin-settings-help-properties-headers-raw = 允许消息属性对话框列出原始标头。关闭时，仅显示解析的摘要。

admin-settings-help-properties-message-enabled = 在消息菜单和快捷方式上提供属性。关闭时，该对话框不可用。

admin-settings-help-properties-message-permanent-delete = 消息对话框是否可以永久删除消息：允许、询问两次或拒绝。合法保留的消息永远不会被删除。通过这种方式删除无法撤消。

admin-settings-help-properties-message-sections = 消息属性对话框的各个部分：摘要、评分、标题、附件、操作。未列出的部分被隐藏。

admin-settings-help-properties-offline-clear-enabled = 允许用户从其“属性”对话框中清除单个文件夹的脱机缓存。

admin-settings-help-properties-remote-location-visibility = 用户看到的服务器端位置的数量：完整（主机和路径）、屏蔽主机（主机部分隐藏）、仅路径（没有主机的邮箱路径）或隐藏。

admin-settings-help-properties-scoring-show = 在消息属性对话框中显示垃圾邮件分数、身份验证结果和安全信号。

admin-settings-help-properties-shortcut-enabled = 从键盘上打开焦点文件夹或选定消息的“属性”对话框。

admin-settings-help-properties-shortcut-key = 打开属性的组合键；默认情况下是 Alt+Enter。

admin-settings-help-reader-actions-archive = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-block-sender = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-colors = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-conversation = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-create-filter = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-delete = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-delete-sender = 默认隐藏。当发件人删除功能和功能配置文件允许时，选择工具栏或更多菜单以启用它。除非锁定，否则用户可以覆盖放置。

admin-settings-help-reader-actions-edit-draft = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-export = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-finish-followup = 选择工具栏、更多菜单或隐藏来完成标记的消息和后续提醒。

admin-settings-help-reader-actions-flag = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-focus-focused = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-focus-other = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-forward = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-headers = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-links = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-move = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-not-spam = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-open-draft = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-pin = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-print = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-properties = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-read = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-reply = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-reply-all = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-save-contacts = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-save-sender = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-search = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-security = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-spam = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-sweep = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-view-source = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-reader-actions-zoom = 选择工具栏、更多菜单或隐藏。功能限制、权限和消息上下文仍然适用。用户可以选择，除非它被锁定。

admin-settings-help-retention-client-errors-event-days = 单个浏览器错误发生（其堆栈跟踪和浏览器类）的保留天数。默认 30。一组保留时间较长，请参阅下一个设置。

admin-settings-help-retention-client-errors-group-days = 浏览器错误组（名称、消息、计数）自上次出现后保留的天数。默认 180。永远不要短于出现保留。

admin-settings-help-retention-metrics-day-buckets-days = 删除前保留的每日性能统计数据天数。 0 使它们不受限制，这使得表在服务器运行期间缓慢增长。

admin-settings-help-retention-metrics-hour-buckets-days = 每小时性能统计数据在合并到每日行（至少 1）之前保留的天数。

admin-settings-help-retention-metrics-minute-buckets-hours = 在将一分钟性能统计数据合并到每小时行（至少 1 个）之前保留的小时数。

admin-settings-help-retention-user-allow-strip-attachments = 用户是否可以创建从旧邮件中删除大附件的规则。默认关闭。关闭时，该操作隐藏在规则编辑器中，使用该操作的规则在保存时会被拒绝，并且在作业运行时会跳过之前保存的规则。

admin-settings-help-retention-user-calendar-enabled = 用户是否可以选择其事件在结束后指定天数被删除的日历。默认关闭。关闭时，该选项隐藏在“设置”中，保存时会拒绝选择，并且不会运行之前保存的选择。

admin-settings-help-retention-user-calendar-max-days = 大多数时候，用户可能会等待已结束的事件被删除。较大的值在保存时会被拒绝，并在运行时会降低。

admin-settings-help-retention-user-calendar-min-days = 事件结束后最少几天用户规则可以将其删除。较小的值在保存时会被拒绝，并在运行时升高。

admin-settings-help-retention-user-notes-enabled = 用户是否可以选择删除在设定的天数内未更改的注释。默认关闭。可以恢复已删除的注释，直到保留规则的撤消窗口结束。

admin-settings-help-retention-user-notes-max-days = 大多数时候，用户可能会等待未触及的笔记被删除。较大的值在保存时会被拒绝，并在运行时会降低。

admin-settings-help-retention-user-notes-min-days = 在用户规则将其删除之前，注释必须保持最少几天不变。较小的值在保存时会被拒绝，并在运行时升高。

admin-settings-help-retention-user-strip-keep-inline-images = 删除附件时是否保留邮件正文中显示的图像。

admin-settings-help-retention-user-strip-min-age-days = 在规则可以删除其附件之前，邮件必须是旧邮件的最少天数。保存规则时会拒绝较小的值，并在运行时引发该值。

admin-settings-help-retention-user-strip-min-attachment-kb = 仅删除至少此大小的附件。较小的附件保留在邮件中。

admin-settings-help-retention-user-strip-undo-days = 删除附件后，原始邮件在“已删除邮件”中保留的天数。 -1 遵循保留规则的撤消窗口。 0 立即删除原始内容。

admin-settings-help-server-shutdown-grace-seconds = 停止的服务器在关闭连接之前允许正在进行的请求完成多长时间（以秒为单位）。 MW_HTTP_SHUTDOWN_TIMEOUT_MS 变量会覆盖它。适用于下次启动。

admin-settings-help-session-last-activity-flush-seconds = 会话的最后查看时间最多以秒为单位写入。零在每个请求上写入它。它不会改变空闲或绝对限制。适用于下次启动。

admin-settings-help-session-persist-across-restarts = 启用后，邮箱、应用程序和控制台会话在服务器重新启动后保持有效。关闭时，每个会话都会在服务器启动时结束。适用于下次启动。

admin-settings-key-attachments-bar-click-to-toggle = 单击附件栏进行切换

admin-settings-key-attachments-bar-download-all = 下载全部按钮

admin-settings-key-attachments-bar-show-download-buttons = 附件下载按钮

admin-settings-key-attachments-bar-zip-inline-images = 下载全部ZIP中的内嵌图像

admin-settings-key-cap-reading-delete-sender = 允许删除发件人的所有邮件

admin-settings-key-compose-feature-family = 作曲家功能

admin-settings-key-compose-headers-allow-auto-submitted = 允许自动提交

admin-settings-key-compose-headers-allow-custom = 允许自定义标头

admin-settings-key-compose-headers-allow-return-receipt-to = 允许回执至

admin-settings-key-compose-headers-allow-threading-edit = 允许编辑线程标题

admin-settings-key-compose-headers-forced = 强制标头

admin-settings-key-compose-insert-image-resize-default = 图片尺寸预设

admin-settings-key-compose-insert-max-inline-image-mb = 最大内嵌图片(MB)

admin-settings-key-compose-layout-default-font = 默认消息字体

admin-settings-key-compose-reply-to-allow-override = 允许更改回复

admin-settings-key-compose-reply-to-show = 回复行

admin-settings-key-compose-sent-folder-fixed-id = 发送副本：固定文件夹

admin-settings-key-compose-sent-folder-mode = 发送副本：文件夹规则

admin-settings-key-compose-sent-folder-per-identity = 发送副本：每个身份的文件夹

admin-settings-key-compose-sent-folder-rules = 发送副本：规则

admin-settings-key-compose-sent-folder-skip-copy = 发送副本：不保存

admin-settings-key-compose-styles-custom = 定制款式

admin-settings-key-compose-styles-default-set = 风格设定

admin-settings-key-compose-styles-max-custom = 每个用户的自定义样式

admin-settings-key-contacts-save-from-message-add-note = 关于从消息中保存的联系人的注释

admin-settings-key-contacts-save-from-message-default-book = 从消息中保存的联系人地址簿

admin-settings-key-contacts-save-from-message-default-scope = 从消息中保存时首先勾选联系人

admin-settings-key-contacts-save-from-message-enabled = 保存消息中的联系人

admin-settings-key-contacts-save-from-message-skip-automated = 保存联系人时跳过自动发件人

admin-settings-key-contacts-save-sender-enabled = 将发件人保存为联系人

admin-settings-key-contacts-save-sender-use-reply-to = 保存列表发件人的回复

admin-settings-key-dsr-admin-step-up = 再次确认后再批准

admin-settings-key-export-default-format = 导出对话框默认格式

admin-settings-key-export-default-scope = 导出对话框默认范围

admin-settings-key-export-dialog-enabled = 导出对话框

admin-settings-key-export-folder-enabled = 文件夹导出

admin-settings-key-export-folder-format = 文件夹导出格式

admin-settings-key-export-folder-include-subfolders = 在导出中包含子文件夹

admin-settings-key-export-folder-max-bytes = 文件夹导出大小限制

admin-settings-key-export-folder-max-messages = 文件夹导出邮件限制

admin-settings-key-export-formats = 导出对话框格式

admin-settings-key-export-message-download-enabled = 留言下载

admin-settings-key-folders-hide-empty = 隐藏空文件夹

admin-settings-key-folders-pinned-hide-from-tree = 从树中隐藏固定的文件夹

admin-settings-key-mail-badge-auth = 认证徽章

admin-settings-key-mail-badge-density = 读者徽章尺寸

admin-settings-key-mail-badge-folder = 文件夹徽章

admin-settings-key-mail-badge-headers = 标题徽章

admin-settings-key-mail-badge-labels = 读者徽章标签

admin-settings-key-mail-badge-list = 邮件列表徽章

admin-settings-key-mail-badge-sanitized = 已消毒HTML徽章

admin-settings-key-mail-badge-spam = 垃圾邮件分数徽章

admin-settings-key-mail-brand-avatars-require-auth = 品牌标志需要认证

admin-settings-key-mail-compose-header-auto-collapse-seconds = Composer 标头自动折叠延迟

admin-settings-key-mail-compose-priority-default = 默认消息优先级

admin-settings-key-mail-compose-priority-enabled = 作曲家中的优先级选择器

admin-settings-key-mail-expiry-hide-expired = 过期消息

admin-settings-key-mail-list-keep = 永远保留

admin-settings-key-mail-list-page-size = 消息列表页面大小

admin-settings-key-mail-list-unsubscribe-mode = 邮件列表 取消订阅

admin-settings-key-mail-message-list-group = 消息分组

admin-settings-key-mail-priority-marks = 优先标记

admin-settings-key-mail-priority-marks-style = 优先标记样式

admin-settings-key-mail-priority-send-headers = 发送时写入的优先级标头

admin-settings-key-mail-priority-trust-sender-headers = 信任发件人设置的优先级

admin-settings-key-mail-reply-from-header-priority = 选择回复身份的标头

admin-settings-key-mail-search-default-scope = 默认邮件搜索范围

admin-settings-key-metrics-dimensions-enabled = 按客户端类型和设备类别划分统计数据

admin-settings-key-metrics-persist-enabled = 在重新启动期间保留统计信息

admin-settings-key-metrics-persist-flush-seconds = 统计保存间隔（秒）

admin-settings-key-observability-client-capture-console = 报告控制台错误

admin-settings-key-observability-client-capture-csp = 举报CSP违规行为

admin-settings-key-observability-client-enabled = 浏览器错误报告

admin-settings-key-observability-client-forward = 转发错误报告

admin-settings-key-observability-client-include-account-hash = 将帐户哈希添加到错误报告中

admin-settings-key-observability-client-max-per-session = 每页加载错误报告

admin-settings-key-observability-client-sample-rate = 错误报告采样率

admin-settings-key-observability-client-source-maps = 使用源映射解析堆栈跟踪

admin-settings-key-privacy-error-reports = 发送错误报告

admin-settings-key-privacy-image-cache-max-stale-secs = 当源发生故障时保持陈旧（秒）

admin-settings-key-privacy-image-cache-stale-secs = 重新验证时服务过时（秒）

admin-settings-key-privacy-image-resize-enabled = 调整大图像的大小

admin-settings-key-privacy-image-resize-format = 调整大小的图像格式

admin-settings-key-privacy-image-resize-max-pixels = 要调整大小的最大图像（像素）

admin-settings-key-privacy-image-resize-max-width = 最宽的调整大小图像（像素）

admin-settings-key-privacy-sessions-show-linked-origin = 会话中链接的邮箱源

admin-settings-key-properties-folder-enabled = 文件夹属性对话框

admin-settings-key-properties-folder-notes-enabled = 文件夹笔记

admin-settings-key-properties-folder-notes-max-length = 文件夹笔记长度

admin-settings-key-properties-folder-overrides-allowed = 允许按文件夹覆盖

admin-settings-key-properties-folder-sections = 文件夹对话框部分

admin-settings-key-properties-folder-size-compute = 文件夹大小计算

admin-settings-key-properties-headers-raw = 原始标头

admin-settings-key-properties-message-enabled = 消息属性对话框

admin-settings-key-properties-message-permanent-delete = 从属性中永久删除

admin-settings-key-properties-message-sections = 消息对话框部分

admin-settings-key-properties-offline-clear-enabled = 清除每个文件夹的离线缓存

admin-settings-key-properties-remote-location-visibility = 远程位置详细信息

admin-settings-key-properties-scoring-show = 评分和认证详细信息

admin-settings-key-properties-shortcut-enabled = 属性快捷方式

admin-settings-key-properties-shortcut-key = 属性快捷键

admin-settings-key-reader-actions-archive = 存档动作放置

admin-settings-key-reader-actions-block-sender = 阻止发件人操作放置

admin-settings-key-reader-actions-colors = 消息颜色操作放置

admin-settings-key-reader-actions-conversation = 查看对话动作放置

admin-settings-key-reader-actions-create-filter = 创建过滤器操作放置

admin-settings-key-reader-actions-delete = 删除动作放置

admin-settings-key-reader-actions-delete-sender = 从发件人操作中删除所有邮件

admin-settings-key-reader-actions-edit-draft = 编辑草稿操作位置

admin-settings-key-reader-actions-export = 导出操作放置

admin-settings-key-reader-actions-finish-followup = 将后续操作标记为已完成操作

admin-settings-key-reader-actions-flag = 标记/取消标记动作放置

admin-settings-key-reader-actions-focus-focused = 移至重点行动位置

admin-settings-key-reader-actions-focus-other = 移至其他操作位置

admin-settings-key-reader-actions-forward = 向前动作放置

admin-settings-key-reader-actions-headers = 原始标题操作放置

admin-settings-key-reader-actions-links = 检查链接操作放置

admin-settings-key-reader-actions-move = 移动动作放置

admin-settings-key-reader-actions-not-spam = 不是垃圾邮件操作放置

admin-settings-key-reader-actions-open-draft = 打开回复草稿操作放置

admin-settings-key-reader-actions-pin = 固定/取消固定动作放置

admin-settings-key-reader-actions-print = 打印动作放置

admin-settings-key-reader-actions-properties = 消息属性操作放置

admin-settings-key-reader-actions-read = 已读/未读操作放置

admin-settings-key-reader-actions-reply = 回复动作放置

admin-settings-key-reader-actions-reply-all = 回复所有动作放置

admin-settings-key-reader-actions-save-contacts = 保存消息联系人操作位置

admin-settings-key-reader-actions-save-sender = 保存发件人操作放置

admin-settings-key-reader-actions-search = 相关搜索操作放置

admin-settings-key-reader-actions-security = 安全模式操作放置

admin-settings-key-reader-actions-spam = 标记为垃圾邮件操作放置

admin-settings-key-reader-actions-sweep = 扫一扫发件人操作放置

admin-settings-key-reader-actions-view-source = 查看消息源操作放置

admin-settings-key-reader-actions-zoom = 缩放动作放置

admin-settings-key-retention-client-errors-event-days = 保留的错误发生次数（天）

admin-settings-key-retention-client-errors-group-days = 保留的错误组（天）

admin-settings-key-retention-metrics-day-buckets-days = 每日统计保存（天）

admin-settings-key-retention-metrics-hour-buckets-days = 每小时保存统计数据（天）

admin-settings-key-retention-metrics-minute-buckets-hours = 保存分钟统计数据（小时）

admin-settings-key-retention-user-allow-strip-attachments = 用户可以从旧邮件中删除附件

admin-settings-key-retention-user-calendar-enabled = 用户可以删除已结束的日历事件

admin-settings-key-retention-user-calendar-max-days = 结束事件之前的大部分天数都会被删除

admin-settings-key-retention-user-calendar-min-days = 结束事件最少几天被删除

admin-settings-key-retention-user-notes-enabled = 用户可以删除未触及的笔记

admin-settings-key-retention-user-notes-max-days = 未触及的笔记被删除之前的大多数日子

admin-settings-key-retention-user-notes-min-days = 未触及的笔记被删除前最少几天

admin-settings-key-retention-user-strip-keep-inline-images = 保留内嵌图像

admin-settings-key-retention-user-strip-min-age-days = 附件被删除前最少几天

admin-settings-key-retention-user-strip-min-attachment-kb = 删除的最小附件 (KB)

admin-settings-key-retention-user-strip-undo-days = 删除附件后的撤消窗口（天）

admin-settings-key-server-shutdown-grace-seconds = 关闭宽限期

admin-settings-key-session-last-activity-flush-seconds = 最后查看的写入间隔

admin-settings-key-session-persist-across-restarts = 重启后保持会话

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-mail-cache-keep-indefinitely = 默认关闭。在浏览器重新加载时保存加密的帐户范围的邮件快照，且不会过期。现有的条目和字节限制仍然适用。用户可以清除已保存的副本；禁用此首选项会将它们从活动浏览器中删除。这不会授予浏览器持久存储权限或下载附件。

admin-settings-help-mail-conversations-allowed = 允许用户将相关回复分组到对话中。默认情况下可用，但用户首选项会启动。禁用可用性会覆盖用户的选择，而不更改类别分组。

admin-settings-help-mail-conversations-enabled = 默认关闭。按线程对加载的回复进行分组，独立于分组依据。设置默认值或锁定此首选项。还必须启用对话可用性。

admin-settings-key-mail-cache-keep-indefinitely = 无限期保留浏览器邮件缓存

admin-settings-key-mail-conversations-allowed = 使对话线程可用

admin-settings-key-mail-conversations-enabled = 将消息分组到对话中

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-links-trusted-sender-domains = 最多 100 个精确小写的发件人域，其已审阅的邮件链接会自动启用。子域是分开的。这不信任目标网站或允许远程图像。默认为空；除非锁定，否则用户可以更改它。

admin-settings-key-links-trusted-sender-domains = 消息链接的可信发件人域

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-reader-attachment-action-style = 在预览标题旁边将下载和文件详细信息显示为图标（默认）、标签或图标和标签。可以全局、每个域或每个用户设置和锁定。

admin-settings-help-reader-attachment-thumbnails = 默认关闭。允许可见图像附件加载缩略图预览。附件安全规则仍然适用。可以全局、每个域或每个用户设置和锁定。

admin-settings-key-reader-attachment-action-style = 附件预览操作

admin-settings-key-reader-attachment-thumbnails = 图像附件缩略图

admin-settings-key-mail-compose-auto-add-mention-recipients = 自动将提及的人添加为收件人
admin-settings-help-mail-compose-auto-add-mention-recipients = 从 @提及建议中选择某人时，如果对方尚未出现在收件人、抄送或密送中，则将其添加到收件人。默认关闭。

admin-settings-linked-disable-title = 要禁用关联账号的已保存访问吗？
admin-settings-linked-disable-body = 保存此更改会删除用于续期关联账号访问权限的凭据。需要续期时，用户必须重新登录关联账号。重新启用此选项不会恢复这些凭据。
admin-settings-linked-disable-confirm = 禁用并清除已保存访问
