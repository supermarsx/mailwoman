# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: reader. Source: ../en/reader.ftl


# -- Machine translation (2026-11): added to match en ---------------------
reader-settings-title = 邮件显示
reader-settings-help = 选择邮件的显示方式。在你选择覆盖之前，将采用管理员的默认设置。邮件内容始终与应用程序隔离。
reader-colorMode = 邮件颜色
reader-colorMode-help = “自动”会在深色主题下调整邮件颜色。“原始”保留发件人的颜色；“适应”使用当前主题；“反转”会反转颜色，同时保留图片颜色。
reader-fontSize = 阅读字号（px）
reader-fontSize-help = 邮件文字的基础字号，范围为 12 到 24 像素。发件人的格式仍可定义标题大小。
reader-lineHeight = 阅读行高
reader-lineHeight-help = 邮件文字的行间距，为字号的 1.2 到 2.2 倍。
reader-contentWidth = 阅读宽度
reader-contentWidth-help = 限制长行以便舒适阅读，或使用整个邮件窗格。
reader-loading = 加载外观
reader-loading-help = 选择柔和的占位线条、文字指示器，或不显示加载指示。屏幕阅读器仍会收到加载状态。
reader-loadingDelayMs = 加载指示延迟（毫秒）
reader-loadingDelayMs-help = 避免快速请求时指示器一闪而过。这只会延迟指示器，绝不会延迟邮件本身。
reader-option-auto = 随主题自动
reader-option-original = 邮件原始颜色
reader-option-adaptive = 适应主题
reader-option-invert = 反转邮件颜色
reader-option-comfortable = 舒适
reader-option-wide = 宽
reader-option-full = 整个窗格
reader-option-skeleton = 占位线条
reader-option-indicator = 文字指示器
reader-option-none = 无视觉指示
reader-inherit = 使用管理员默认设置
reader-actionStyle = 邮件操作功能区
reader-actionStyle-help = 紧凑图标可为邮件留出更多空间。每个操作都保留键盘名称和说明性的工具提示。如果你喜欢图标旁边带文字，请选择显示标签。
reader-option-icons = 紧凑图标
reader-option-labels = 图标和标签
reader-zoom-group = 邮件缩放
reader-zoom-in = 放大
reader-zoom-in-help = 放大邮件。只有邮件的大小会改变。
reader-zoom-out = 缩小
reader-zoom-out-help = 缩小邮件。只有邮件的大小会改变。
reader-zoom-level = { $level }%
reader-zoom-reset-label = 邮件缩放 { $level }%。重置为 100%
reader-zoom-reset-help = 将邮件重置为 100%
reader-zoom-reset-menu = 重置缩放（{ $level }%）
reader-zoom = 邮件缩放（%）
reader-zoom-help = 邮件打开时的大小，范围为 50% 到 200%。只有邮件正文会缩放，应用本身不会，打印时始终使用 100%。邮件操作旁边的缩放按钮也可以更改此设置。
reader-attachments-summary = { $count ->
   *[other] { $count } 个附件
} · { $size }
reader-attachments-show = 显示附件
reader-attachments-hide = 隐藏附件
reader-attachmentsDensity = 附件密度
reader-attachmentsDensity-help = “紧凑”显示小型文件按钮；“舒适”在展开时显示更大的预览。
reader-option-compact = 紧凑
reader-attachmentsCollapsed = 默认折叠附件
reader-attachmentsCollapsed-help = 每封邮件默认以紧凑的附件摘要开始。你可以展开某封邮件而不改变此默认设置。
reader-card-text = 文字和颜色
reader-card-text-help = 邮件文字的大小、间距、宽度和颜色。
reader-card-actions = 操作和附件
reader-card-actions-help = 邮件操作功能区，以及附件的初始状态。
reader-card-loading = 加载
reader-card-loading-help = 邮件打开时你看到的内容。
settings-mail-card-list = 邮件列表
settings-mail-card-list-help = 行操作，以及对邮件执行操作时列表的行为。

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
reader-attachments-download = 下载{ $name }

reader-attachments-download-all = 下载全部

reader-attachments-download-all-blocked = 附件策略阻止此处的每个文件

reader-attachments-download-all-busy = 准备ZIP文件

reader-attachments-download-all-label = 将所有附件下载为 ZIP 文件

reader-attachments-download-all-none = 没有附件可供下载

reader-attachments-download-all-warned = { $count ->
       *[other] 遗漏了 { $count } 个需要确认的文件。将它们一一下载。
    }

reader-attachments-download-blocked = { $name }无法下载：{ $reason }

reader-attachments-download-label = 下载{ $name } ({ $size })

reader-attachments-download-off = 下载已关闭

reader-attachments-inline-group = { $count ->
       *[other] { $count } 个内联图像
    }

reader-attachments-inline-summary = { $count ->
       *[other] { $count } 个内联图像
    } · { $size }

reader-attachmentsBarClickToToggle = 单击该栏可展开或折叠

reader-attachmentsBarClickToToggle-help = 单击附件栏上的任意位置可对其进行切换。关闭时，只有箭头按钮起作用。

reader-attachmentsBarDownloadAll = 下载全部按钮

reader-attachmentsBarDownloadAll-help = 在栏上显示一个按钮，将附件下载为一个 ZIP 文件。

reader-attachmentsBarShowDownloadButtons = 附件上的下载按钮

reader-attachmentsBarShowDownloadButtons-help = 在悬停或聚焦时显示每个附件的下载按钮，始终或从不。

reader-attachmentsBarZipInlineImages = 在 ZIP 中包含内嵌图像

reader-attachmentsBarZipInlineImages-help = 除非打开此选项，否则消息文本中出现的图像将不会被全部下载。

reader-card-attachments-bar = 附件栏

reader-card-attachments-bar-help = 邮件附件上方的栏如何响应以及它提供什么。标记为锁定的线路由管理员设置。

reader-option-always = 总是

reader-option-hover = 悬停或聚焦时

reader-option-never = 从来没有

reader-card-header = 邮件标头
reader-autoHideHeader = 自动隐藏邮件标头
reader-autoHideHeader-help = 打开每封邮件时折叠发件人、收件人和元数据详情。主题和操作保持可见。使用显示标头可查看详情。默认关闭。
reader-header-show = 显示标头
reader-header-hide = 隐藏标头
