# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: retention. Source: ../en/retention.ftl


# -- Machine translation (2026-11): added to match en ---------------------
retention-title = 保留规则
retention-intro = 在经过指定天数后移动或删除你自己邮件的规则。规则可以根据文件夹、标签、发件人、大小、是否已读或是否有附件来判断。已加旗标的邮件和带有 keep 标签的邮件默认不受影响，除非你更改设置。删除的邮件会先进入废纸篓，并在撤销期限过后被清除。
retention-disabled = 此服务器已关闭保留规则。你现有的规则不会运行。
retention-never = 从不
retention-error = 无法加载或保存保留设置。请重试。
retention-last-run = 上次运行：{ $when }
retention-next-run = 约 { $days } 天后下次运行
retention-next-run-unknown = 尚未安排下次运行
retention-enabled = 运行我的保留规则
retention-trash-first = 将删除的邮件先移至废纸篓
retention-trash-forced = 此服务器始终先将邮件移至废纸篓，并在撤销期限过后清除。
retention-undo = 撤销期限（天）
retention-undo-help = 被规则移至废纸篓的邮件会在其中保留这么久，然后被清除。0 至 { $max } 天。
retention-pause = 暂停所有规则 30 天
retention-paused-until = 已暂停，至 { $when }
retention-resume = 恢复
retention-policy = 此服务器允许的操作
retention-limits-both = 删除邮件的规则至少等待 { $min } 天，至多等待 { $max } 天。
retention-limits-min = 删除邮件的规则至少等待 { $min } 天。
retention-max-rules = 你最多可以保留 { $count } 条规则。
retention-permanent-allowed = 关闭“废纸篓”步骤后，规则可以永久删除邮件。
retention-permanent-denied = 规则不能永久删除邮件。邮件会进入废纸篓，并在 { $days } 天后被清除。
retention-protected = 规则不会处理的文件夹：{ $folders }。
retention-mandatory = 由管理员设置：{ $folder } 中超过 { $days } 天的邮件将被删除。
retention-rules = 你的规则
retention-add = 新建规则
retention-no-rules = 你还没有规则。可使用下方的模板，或添加一条。
retention-rule-paused = 已暂停
retention-rule-on = 开
retention-rule-enabled = 规则 { $name } 已开启
retention-edit-short = 编辑
retention-delete = 删除
retention-new = 新建规则
retention-edit = 编辑规则
retention-name = 名称
retention-action = 执行操作
retention-action-keep = 永久保留
retention-action-delete = 删除
retention-action-trash = 移至废纸篓
retention-action-archive = 移至归档
retention-action-move = 移至文件夹
retention-action-markRead = 标记为已读
retention-days = 天数
retention-target = 文件夹
retention-choose-folder = 选择文件夹
retention-scope = 适用的邮件
retention-folder = 文件夹
retention-any-folder = 任意文件夹
retention-subfolders = 包含子文件夹
retention-labels = 标签（以逗号分隔）
retention-senders = 发件人地址（以逗号分隔）
retention-domains = 发件人域名（以逗号分隔）
retention-min-size = 最小大小 (KB)
retention-max-size = 最大大小 (KB)
retention-read = 阅读状态
retention-read-any = 已读或未读
retention-read-read = 仅已读
retention-read-unread = 仅未读
retention-attachment = 附件
retention-attachment-any = 有无均可
retention-attachment-with = 有附件
retention-attachment-without = 无附件
retention-exclude = 不处理
retention-exclude-flagged = 已加旗标（加星）的邮件
retention-exclude-unread = 未读邮件
retention-exclude-tags = 带有这些标签的邮件（以逗号分隔）
retention-exclude-note = 处于法律保留状态的邮件无论此处如何设置，都不会被处理。
retention-role-inbox = 收件箱
retention-role-sent = 已发送
retention-role-archive = 归档
retention-role-junk = 垃圾邮件
retention-role-trash = 废纸篓
retention-role-drafts = 草稿
retention-btn-preview = 预览
retention-save = 保存
retention-cancel = 取消
retention-preview-none = 没有邮件会被更改。
retention-preview-count = 这将更改 { $count } 封邮件。
retention-preview-truncated = 此预览只读取了邮箱的一部分。
retention-preview-sample = 示例
retention-no-subject = （无主题）
retention-do-trash = 移至废纸篓
retention-do-move = 已移动
retention-do-delete = 已删除
retention-do-markRead = 已标为已读
retention-skipped-legalHold = 你的账户处于法律保留状态。不会进行任何更改。
retention-skipped-restricted = 你的账户处理已受限。不会进行任何更改。
retention-skipped-noRules = 你没有启用的规则，管理员也未设置规则。
retention-error-name = 请为规则命名。
retention-error-days = 天数必须是 0 到 3650 之间的整数。
retention-error-min = 删除邮件的规则至少要等待 { $days } 天。
retention-error-max = 删除邮件的规则最多只能等待 { $days } 天。
retention-error-target = 请选择邮件要移至的文件夹。
retention-summary-keep = 永久保留
retention-summary-delete = { $days } 天后移至废纸篓，随后清除
retention-summary-delete-permanent = { $days } 天后永久删除
retention-summary-trash = { $days } 天后移至废纸篓
retention-summary-archive = { $days } 天后移至归档
retention-summary-move = { $days } 天后移至文件夹
retention-summary-markRead = { $days } 天后标为已读
retention-templates = 模板
retention-templates-help = 模板会添加一条现成的规则。使用前请检查其标签和天数。
retention-adopt = 添加此规则
retention-run-heading = 立即运行
retention-preview-all = 预览所有规则
retention-btn-run = 立即运行我的规则
retention-run-result = 已更改 { $count } 封邮件。
retention-history = 最近运行
retention-history-when = 时间
retention-history-matched = 匹配
retention-history-changed = 已更改
retention-history-errors = 错误
retention-context-action = 保留规则…

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
pim-cal-calendars = 日历

pim-cal-days = 活动结束后几天

pim-cal-enabled = 删除已结束的事件

pim-cal-help = 在日历中选择的活动结束后指定天数后将其删除。重复发生的事件永远不会被删除。

pim-cal-title = 日历事件

pim-days-range = 在{ $min }和{ $max }天之间选择。

pim-held-title = 已删除的笔记可以恢复

pim-held-until = 保留到{ $when }

pim-limits = { $min } 到{ $max } 天之间。

pim-notes-days = 自上次更改注释以来的天数

pim-notes-enabled = 删除未触及的笔记

pim-notes-help = 删除您在指定天数内未更改的注释。保留固定的笔记。删除的笔记可以恢复{ $days }天。

pim-notes-title = 注释

pim-off = 已被您的管理员关闭。

pim-preview = 预览

pim-restore = 恢复

pim-result = { $events } 事件和{ $notes } 注释。

pim-run = 立即运行

pim-run-title = 立即运行

pim-save = 保存

pim-skipped = { $count } 选项被跳过，因为您的管理员将其关闭。

pim-untitled = 无标题笔记

retention-action-stripAttachments = 删除大附件

retention-do-stripAttachments = 附件已删除

retention-error-strip-disabled = 您的管理员禁止删除附件。

retention-error-strip-min = 删除附件的规则必须等待至少{ $days }天。

retention-fd-action = { $folder } 中的邮件会发生什么情况

retention-fd-days = 处理 { $folder } 之前的天数

retention-fd-help = 设置邮件在移动或删除之前在文件夹中保留的时间。设置为“继承”的文件夹没有自己的默认值；上述规则和管理员限制仍然适用。子文件夹位于其文件夹后面。

retention-fd-inherit = 继承

retention-fd-mandatory = 管理员将此文件夹保留{ $days }天

retention-fd-protected = 受管理员保护

retention-fd-save = 保存文件夹默认值

retention-fd-title = 文件夹默认值

retention-rule-strip-off = 已被您的管理员关闭

retention-strip-allowed = 规则可以从至少 { $days } 天前的邮件中删除至少 { $kb } KB 的附件。每个附件均由简短的注释代替。

retention-strip-denied = 您的管理员已关闭删除附件功能。

retention-strip-done = 从{ $count }邮件中删除附件，释放{ $size }。

retention-strip-inline-kept = 消息内显示的图像将被保留。

retention-strip-inline-removed = 消息中显示的图像也会被删除。

retention-strip-limits = 从至少 { $days } 天前的邮件中删除至少 { $kb } KB 的附件。原件将在垃圾箱中保留{ $undo }天。

retention-strip-preview = { $count }消息将丢失附件，释放{ $size }。

retention-strip-preview-estimate = 大约 { $count } 封邮件将丢失附件，最多释放 { $size }。

retention-strip-skipped = { $count } 删除附件的规则已被跳过，因为您的管理员已将其关闭。

retention-summary-stripAttachments = { $days }天后，大附件被删除
