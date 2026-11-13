# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: mail. Source: ../en/mail.ftl

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = 设置
mail-compose = 撰写
mail-nav-mailboxes = 邮箱
mail-nav-attachments = 附件
mail-nav-outbox = 发件箱
mail-nav-sharing = 共享文件夹
mail-nav-apps = 应用
mail-logout = 退出登录
mail-offline = 离线
mail-module-loading = 正在加载 { $module }…

# -- Search ------------------------------------------------------------------
mail-search = 搜索
mail-search-label = 搜索邮件
mail-search-placeholder = 搜索邮件——from:alice subject:invoice larger:1mb
mail-search-clear = 清除

# -- Message list ------------------------------------------------------------
mail-list-label = 邮件
mail-loading = 正在加载邮件…
mail-empty = 没有邮件
mail-unknown-sender = （未知发件人）
mail-no-subject = （无主题）
mail-pinned = 已置顶
mail-has-attachment = 有附件
mail-answered = 已回复
mail-forwarded = 已转发
mail-unread = 未读
mail-flagged = 已加旗标
# Announced on a selected row; the row also shows a check mark.
mail-selected = 已选择
# Announced row summary for screen readers (position within the whole list).
mail-row-position = 第 { $pos } 封邮件，共 { $total } 封

# -- Conversation threading (W2) ---------------------------------------------
mail-thread-count = { $count ->
   *[other] { $count } 封邮件
}
mail-thread-expand = 展开会话，{ $count ->
   *[other] { $count } 封邮件
}
mail-thread-collapse = 折叠会话，{ $count ->
   *[other] { $count } 封邮件
}

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = 视图选项
mail-reading-pane = 阅读窗格
mail-reading-pane-right = 右侧
mail-reading-pane-bottom = 底部
mail-reading-pane-off = 关闭

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = 已选择的邮件
mail-bulk-count = 已选择 { $count } 封
mail-mark-read = 标为已读
mail-mark-unread = 标为未读
# Sets the message's IMAP flag (\Flagged). Not the same as a follow-up reminder.
mail-flag-message = 加旗标
mail-unflag-message = 移除旗标
mail-move-to = 移动到文件夹
mail-move-to-placeholder = 移动到…
mail-select-all = 全选
mail-clear-selection = 清除选择

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-filter = 收件箱筛选
mail-inbox-focused = 重点
mail-inbox-other = 其他

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = 已打开的标签页
mail-subtab-pin = 固定 { $title }
mail-subtab-unpin = 取消固定 { $title }
mail-subtab-tearoff = 在新窗口中打开 { $title }
mail-subtab-close = 关闭 { $title }

# -- Per-row actions ---------------------------------------------------------
mail-pin = 置顶
mail-unpin = 取消置顶
mail-snooze = 稍后提醒
mail-snooze-menu = 稍后提醒时间
mail-snooze-later = 今天晚些时候
mail-snooze-tomorrow = 明天
mail-snooze-next-week = 下周
mail-unsnooze = 取消稍后提醒
mail-label = 标签
mail-labels-menu = 标签
mail-flag = 标记为待跟进
mail-clear-flag = 清除待跟进
mail-archive = 归档
mail-delete = 删除
# Opens the row's actions where they cannot be revealed by hovering (touch, phone).
mail-more-actions = 更多操作

# -- Reader ------------------------------------------------------------------
mail-reader-label = 邮件
mail-reader-empty = 选择一封邮件以阅读
mail-back = 返回
mail-reader-from = 发件人：{ $addr }
mail-reader-to = 收件人：{ $addr }
mail-reader-cc = 抄送：{ $addr }
mail-reader-actions = 邮件操作
mail-reply = 回复
mail-reply-all = 全部回复
mail-forward = 转发
mail-reply-failed = 无法准备邮件。请重试。
mail-reply-unthreaded = 无法读取原始邮件的 ID。此回复不会与原始邮件归入同一会话。
mail-spam = 垃圾邮件
mail-export = 导出
mail-sweep-sender = 清理发件人邮件
mail-message-body = 邮件正文
mail-no-content = 无内容
mail-sanitizing = 正在净化…
mail-attachments = 附件
mail-attachment-unnamed = （未命名）
mail-attachment-open = 附件 { $name }
mail-attachment-close = 关闭附件
mail-attachment-loading = 正在加载附件…

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = 加密邮件
mail-encrypted-note = 此邮件已端到端加密。请在此设备上解锁后阅读。
mail-key-passphrase = 密钥密码短语
mail-decrypt = 解密
mail-decrypting = 正在解密…
mail-decrypt-no-key = 没有可用于解密此邮件的私钥。
mail-decrypt-failed = 解密失败

# -- Reply and forward: text written into the outgoing message --------------
# The line above the quoted original of a reply.
mail-quote-attribution = { $name } 于 { $date } 写道：
mail-quote-attribution-undated = { $name } 写道：
# The header block above a forwarded original.
mail-forward-heading = ---------- 转发的邮件 ----------
mail-forward-from = 发件人：{ $addr }
mail-forward-date = 日期：{ $date }
mail-forward-subject = 主题：{ $subject }
mail-forward-to = 收件人：{ $addr }
mail-forward-cc = 抄送：{ $addr }

# -- Composer ----------------------------------------------------------------
mail-compose-label = 撰写邮件
mail-compose-title = 新邮件
mail-compose-title-reply = 回复
mail-compose-title-reply-all = 全部回复
mail-compose-title-forward = 转发
mail-compose-close = 关闭
mail-compose-from = 发件人
mail-compose-from-default = 默认
mail-compose-to = 收件人
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = 联系人建议
mail-compose-cc = 抄送
mail-compose-bcc = 密送
mail-compose-show-cc-bcc = 添加抄送或密送
mail-compose-bcc-encrypted = 加密邮件会列出每个收件人的密钥，因此密送收件人会被其他收件人看到。请移除密送收件人或关闭加密。
mail-compose-plaintext-confirm-title = 不加密发送已解密的文本？
mail-compose-plaintext-confirm-body = 此邮件引用了一封加密邮件中的文本。此邮件未设置为加密，因此引用的文本将在无保护的情况下发送。
mail-compose-plaintext-confirm-send = 不加密发送
mail-compose-plaintext-confirm-cancel = 返回
mail-compose-subject = 主题
mail-compose-body = 正文
mail-compose-attachments = 附件
mail-compose-remove-attachment = 移除 { $name }
mail-compose-attach-nextcloud = 从 Nextcloud 添加附件
mail-compose-close-nextcloud = 关闭 Nextcloud
# New-file blob upload (attach a local file from this device)
mail-compose-attach-file = 添加文件附件
mail-compose-uploading = 正在上传…
mail-compose-upload-unavailable = 当前无法上传文件。
mail-compose-upload-failed = 无法上传 { $name }。
mail-compose-upload-too-large = { $name } 大小为 { $size } MB。上传大小上限为 { $max } MB。
mail-compose-send-later = 稍后发送
mail-compose-cancel = 取消
mail-compose-send = 发送
mail-compose-schedule = 定时发送
mail-compose-sending = 正在发送…
mail-compose-encrypted-subject = 加密邮件
mail-compose-dlp-blocked = 发送已被数据防泄漏规则阻止（请参阅上方的警告）。
mail-compose-send-failed = 发送失败
mail-compose-send-later-past = 请选择将来的发送时间，或清空此字段以立即发送。
mail-compose-options = 发送选项
mail-compose-request-receipt = 请求已读回执
mail-compose-request-receipt-hint = 收件人的邮件程序可能会询问收件人、不询问直接发送回执，或忽略此请求。
mail-compose-track-opens = 记录此邮件何时被打开
mail-compose-track-opens-hint = 邮件中会添加一张从本服务器加载的 1×1 图片。许多邮件程序会屏蔽图片或提前加载图片，因此记录到打开并不能证明邮件已被阅读，没有记录也不能证明未被阅读。收件人不会收到通知。
mail-compose-image-type = { $name } 无法放入正文。请使用 PNG、JPEG、GIF 或 WebP 图片。
# Refusals raised by the send itself. The composer stays open and shows them.
mail-send-not-signed-in = 你尚未登录。
mail-send-no-draft-folder = 此账户没有用于保存草稿的文件夹。
mail-send-no-recipient = 请至少添加一个收件人。
mail-send-bad-recipient = { $count ->
   *[other] 以下不是有效的电子邮件地址：{ $addresses }
}
mail-send-later-past = 发送时间不在将来。未发送任何内容。
mail-send-tracking-failed = 目前无法记录打开情况，因此未发送任何内容。取消勾选“记录此邮件何时被打开”即可不带此功能发送。
mail-send-draft-refused = 服务器拒绝了此邮件：{ $reason }。未发送任何内容。
mail-send-submission-refused = 服务器拒绝发送此邮件：{ $reason }。未发送任何内容。
# Signing-key unlock (sign-on-send)
mail-compose-sign-unlock-title = 解锁签名密钥
mail-compose-sign-unlock-note = 输入密钥密码短语以对此邮件签名。密钥仅在此撰写窗口中保持解锁状态。
mail-compose-sign-unlock = 解锁
mail-compose-sign-unlocking = 正在解锁…
mail-compose-sign-no-key = 此设备上没有可用的签名密钥。
mail-compose-sign-unlock-failed = 无法解锁签名密钥。
mail-compose-sign-unlock-required = 请解锁签名密钥以发送此签名邮件。

# Rich-text editor (W1)
mail-compose-format-plain = 纯文本
mail-compose-format-rich = 富文本
mail-compose-rt-toolbar = 格式
mail-compose-rt-bold = 粗体
mail-compose-rt-italic = 斜体
mail-compose-rt-underline = 下划线
mail-compose-rt-strike = 删除线
mail-compose-rt-bullet = 项目符号列表
mail-compose-rt-ordered = 编号列表
mail-compose-rt-quote = 引用
mail-compose-rt-link = 链接
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = 应用
mail-compose-rt-image = 插入图片

# Signature picker (W12)
mail-compose-signature = 签名
mail-compose-signature-none = 添加签名…

# Recall (W10)
mail-compose-recall = 撤回
mail-compose-recall-action = 撤回
mail-compose-recall-holding = 已暂存，等待发送
mail-compose-recall-scheduled = 定于 { $when } 发送

# Drafts drawer (W9)
mail-compose-drafts = 草稿
mail-compose-drafts-empty = 没有已保存的草稿
mail-compose-drafts-delete = 删除草稿
mail-compose-drafts-no-recipient = （无收件人）

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = 发件箱
mail-outbox-refresh = 刷新
mail-outbox-empty = 没有等待发送的邮件
mail-outbox-send-now = 立即发送
mail-outbox-cancel = 取消
mail-outbox-scheduled = 已定时
mail-outbox-holding = 即将发送
mail-outbox-sent = 已发送
mail-outbox-canceled = 已取消
# A held submission is one the server will not send until the mailbox owner
# releases it here (EmailSubmission `mailwomanHold: "manual"`). $name is the
# API key's prefix, or the connected app's client id.
mail-outbox-held = 已暂扣
mail-outbox-failed = 未发送
mail-outbox-release = 放行
mail-outbox-discard = 丢弃
mail-outbox-held-note = 已暂扣。放行之前不会发送。
mail-outbox-held-note-api-key = 已暂扣——由 API 密钥 { $name } 创建。放行之前不会发送。
mail-outbox-held-note-oauth = 已暂扣——由已连接的应用 { $name } 创建。放行之前不会发送。
mail-outbox-origin-api-key = 由 API 密钥 { $name } 创建
mail-outbox-origin-oauth = 由已连接的应用 { $name } 创建
mail-outbox-to = 收件人：{ $recipients }
mail-outbox-no-subject = （无主题）
mail-outbox-attempts = 已尝试 { $count } 次。最近一次错误：{ $error }
mail-outbox-error = 错误：{ $error }
mail-outbox-toast-canceled = 已取消发送
mail-outbox-toast-discarded = 已丢弃。邮件未发送。
mail-outbox-toast-sent = 已发送
mail-outbox-toast-retrying = 尚未发送：{ $error }。稍后将重试。
mail-outbox-toast-failed = 未发送：{ $error }
mail-outbox-toast-not-released = 无法发送：{ $error }
mail-outbox-toast-not-canceled = 无法取消：{ $error }

# -- Network status and the offline queue ------------------------------------
# Raised by the store (state/store.ts, state/slices/offline.ts) and listed by
# the notice above the message list (components/OfflineQueueNotice.tsx).
mail-net-lost = 连接已断开——正在重试…
mail-net-back = 已恢复在线
mail-queue-sent = { $count ->
   *[other] 已发送 { $count } 项排队的更改
}
mail-queue-failed = { $count ->
   *[other] { $count } 项排队的更改失败
}
mail-queue-notice-label = 无法应用的更改
mail-queue-notice-title = { $count ->
   *[other] { $count } 项离线时所做的更改无法应用
}
mail-queue-kind-send = 发送“{ $subject }”
mail-queue-kind-send-untitled = 发送一封无主题的邮件
mail-queue-kind-move = 移动邮件
mail-queue-kind-flag = 更改标签或旗标
mail-queue-kind-draft = 保存草稿
mail-queue-kind-pim = 日历、任务、笔记或联系人更改
mail-queue-reason = 原因：{ $reason }
mail-queue-reason-unknown = 服务器未说明原因。
mail-queue-retry = 重试
mail-queue-discard = 丢弃
mail-queue-retry-item = 重试：{ $what }
mail-queue-discard-item = 丢弃：{ $what }
mail-queue-discarded = 已丢弃。该更改未应用。

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = 清理邮件
mail-sweep-title = 清理 { $sender } 的邮件
mail-sweep-all = 删除此发件人的所有邮件
mail-sweep-keep-latest = 保留最新的一封，删除其余邮件
mail-sweep-older-than = 删除早于 N 天的邮件
mail-sweep-block = 全部删除并阻止此发件人
mail-sweep-days = 天数
mail-sweep-count = { $count ->
   *[other] { $count } 封邮件将移至废纸篓
}
mail-sweep-cancel = 取消
mail-sweep-run = 清理 { $count } 封
mail-sweeping = 正在清理…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = 关闭
mail-undo-action = 撤销
mail-undo-send-cancel = 取消
mail-undo-failed = 无法撤销

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = 已添加标签
mail-toast-label-removed = 已移除标签
mail-toast-pinned = 已置顶邮件“{ $target }”
mail-toast-unpinned = 已取消置顶邮件“{ $target }”
mail-toast-snoozed = 已设置稍后提醒
mail-toast-follow-up-set = 已设置待跟进
mail-toast-follow-up-cleared = 已清除待跟进
mail-toast-archived = 已归档
mail-toast-trashed = 已移至废纸篓
mail-toast-marked-spam = 已标记为垃圾邮件
mail-toast-moved-to = 已移至 { $folder }
mail-toast-moved = 已移至文件夹
mail-toast-archived-count = 已归档 { $count } 封邮件
mail-toast-trashed-count = 已将 { $count } 封邮件移至废纸篓
mail-toast-marked-spam-count = 已将 { $count } 封邮件标记为垃圾邮件
mail-toast-moved-count = 已移动 { $count } 封邮件
mail-toast-moved-to-count = 已将 { $count } 封邮件移至 { $folder }
mail-toast-marked-read = { $count ->
   *[other] 已将 { $count } 封邮件标为已读
}
mail-toast-marked-unread = { $count ->
   *[other] 已将 { $count } 封邮件标为未读
}
mail-toast-flagged = { $count ->
   *[other] 已为 { $count } 封邮件加旗标
}
mail-toast-unflagged = { $count ->
   *[other] 已移除 { $count } 封邮件的旗标
}
mail-toast-move-failed = 无法移动邮件。未移动任何邮件。
mail-toast-update-failed = 无法更改邮件。未更改任何内容。
mail-toast-update-refused = { $count ->
   *[other] 服务器拒绝了对 { $count } 封邮件的更改
}
mail-toast-no-archive-folder = 没有归档文件夹
mail-toast-no-trash-folder = 没有废纸篓文件夹
mail-toast-no-spam-folder = 没有垃圾邮件文件夹
mail-toast-sweep-nothing = 没有可清理的邮件
mail-toast-swept = { $count ->
   *[other] 已清理 { $count } 封邮件
}
mail-toast-load-more-failed = 无法加载更多邮件
mail-toast-export-nothing = 没有可导出的内容
mail-toast-exported = 已导出 .eml
mail-toast-export-failed = 导出失败
mail-toast-send-queued = 已排队——恢复在线后将发送
mail-toast-tracking-not-bound = 邮件已发送，但不会记录其打开情况。
mail-toast-send-scheduled = 已定时发送
mail-toast-sent = 邮件已发送
mail-toast-send-canceled = 已取消发送

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = 功能区
mail-ribbon-tab-home = 开始
mail-ribbon-tab-view = 视图
mail-ribbon-tab-folder = 文件夹
mail-ribbon-expand = 展开
mail-ribbon-collapse = 折叠
mail-ribbon-group-new = 新建
mail-ribbon-compose = 撰写
mail-ribbon-group-session = 会话
mail-ribbon-logout = 退出登录
mail-ribbon-group-theme = 主题
mail-ribbon-group-density = 密度
mail-ribbon-group-settings = 设置
mail-ribbon-more = 更多…
mail-ribbon-group-folders = 文件夹
mail-density-compact = 紧凑
mail-density-cozy = 适中
mail-density-relaxed = 宽松

# -- Message security level: the three-position switch in the reader toolbar
# (viewers/MaxSecuritySwitch.tsx). The hint is the tooltip of each position.
mail-maxsec-label = 邮件安全级别
mail-maxsec-full = 完整
mail-maxsec-full-hint = 经过净化的 HTML，包含图片和样式。
mail-maxsec-no-media = 无媒体
mail-maxsec-no-media-hint = 经过净化的 HTML，已移除所有图片和媒体。
mail-maxsec-plain-text = 纯文本
mail-maxsec-plain-text-hint = 不渲染 HTML——邮件仅以纯文本显示。
mail-feature-off = 已被你的设置或管理员关闭
mail-send-disabled = 发送功能已被你的设置或管理员关闭。你仍可以保存草稿。

# -- Machine translation (2026-11): added to match en ---------------------
mail-nav-events = 活动
mail-events-upcoming = 仅显示即将举行
mail-events-filter = 活动筛选
mail-loading-more = 正在加载更早的邮件…
mail-event-state-invitation = 邀请
mail-event-state-updated = 已更新
mail-event-state-cancelled = 已取消
mail-event-state-reply = 回复
mail-event-state-counter = 已提议新时间
mail-event-state-event = 活动
mail-event-hint = { $state }，{ $when }
mail-row-actions-expand = 显示更多操作
mail-remote-moved = 此邮件已在其他设备或应用中移至 { $folder }。你仍在阅读打开时的副本。
mail-remote-moved-unknown = 此邮件已在其他设备或应用中被移动。你仍在阅读打开时的副本。
mail-remote-deleted = 此邮件已在其他设备或应用中被删除。在关闭之前，你可以继续阅读此副本。
mail-remote-dismiss = 关闭
mail-reply-draft = 回复进行中
mail-reply-draft-open = 打开草稿
mail-draft-load-failed = 无法打开草稿。请重试。
mail-reply-images-omitted =
    { $count ->
       *[other] 有 { $count } 张图片无法包含在回复中。
    }
mail-export-forbidden = 管理员已为此账户禁用邮件导出。
mail-export-open-failed = 无法打开导出选项。请关闭一个工作区标签页后重试。
mail-headers-title = 原始邮件头
mail-headers-action-help = 查看原始邮件头，包括投递和身份验证详情。
mail-headers-help = 邮件的原始邮件头。折行会被保留；此处仅显示邮件头。
mail-headers-loading = 正在加载邮件头…
mail-headers-failed = 无法加载邮件头。请检查网络连接后重试。
mail-headers-copy = 复制邮件头
mail-headers-copied = 邮件头已复制。
mail-headers-copy-failed = 无法复制邮件头。请选中文本后手动复制。
mail-attachment-failed = 无法下载此附件。请在网络可用时重试。
mail-compose-rt-rule = 水平线
mail-compose-signature-new = 新邮件
mail-compose-signature-replies = 回复
mail-editor-body-size = { $size } KiB
mail-editor-body-size-label = 正文大小
mail-editor-body-size-help = 当前正文按 UTF-8 计算的大小，包括 HTML 标记和嵌入的图片。不包括邮件头和单独的附件。
mail-outbox-load-failed = 无法检查发件箱。请检查网络连接后点击“刷新”。
mail-outbox-unavailable = 发件箱不可用。请使用“刷新”重试。
mail-maxsec-view = 视图：{ $mode }
mail-maxsec-menu = 视图模式
mail-toast-tab-pinned = 已固定标签页“{ $target }”
mail-toast-tab-unpinned = 已取消固定标签页“{ $target }”
mail-workspace-message = 邮件
mail-opened-announcement = 已打开：{ $subject }
mail-workspace-layout = 工作区布局
mail-workspace-tabs = 标签页
mail-workspace-side-by-side = 并排
mail-workspace-stack = 堆叠
mail-workspace-refresh = 刷新文件夹
mail-workspace-load-error = 无法加载此邮件。请关闭标签页后重试。
mail-resize-folders = 调整文件夹侧边栏大小
mail-resize-list = 调整邮件列表大小
mail-resize-tiles = 调整工作区窗格大小
mail-show-folders = 显示文件夹侧边栏
mail-hide-folders = 隐藏文件夹侧边栏
mail-compact-folders = 紧凑文件夹图标
mail-toggle-list = 显示或隐藏邮件列表
mail-toggle-reader = 显示或隐藏阅读窗格
mail-folder-counts = { $unread } 封未读，共 { $total } 封邮件
mail-print = 打印 / 另存为 PDF
mail-go-to-top = 返回顶部
mail-print-help = 打印此邮件，或在打印对话框中选择“另存为 PDF”。外部图片仍会被阻止。
mail-file-details = 文件详情和指纹
mail-file-type = 文件类型
mail-file-extension = 扩展名
mail-file-detected = 检测到的签名
mail-file-unknown = 无法识别的签名
mail-file-size = 大小
mail-file-size-bytes = { $count } 字节
mail-file-hash-unavailable = 指纹不可用（需要安全的浏览器，且文件不超过 64 MiB）。
mail-file-fingerprint-help = 指纹用于标识这些确切的字节。它并不能证明文件是安全的，也不能证明是谁创建的。
mail-file-warning-executable = 此文件可能运行程序或更改系统设置。仅在信任其来源时才下载。
mail-file-warning-macro = 此文档可能包含宏。除非已确认发件人和用途，否则请勿启用宏。
mail-file-warning-disguisedName = 文件名可能伪装了其真实扩展名。下载前请先查看文件详情。
mail-file-warning-typeMismatch = 检测到的文件签名与其声明的类型不符。自动预览已被阻止。
mail-file-warning-activeDocument = 此格式可能包含活动内容。其预览是隔离的；打开下载的副本时行为可能不同。
mail-file-preview-blocked = 由于此文件类型或类型不符，已禁用自动预览。
mail-quote-header-from = 发件人：
mail-quote-header-sent = 发送时间：
mail-quote-header-to = 收件人：
mail-quote-header-cc = 抄送：
mail-quote-header-subject = 主题：
mail-spam-score = 垃圾邮件分析
mail-spam-score-loading = 正在检查垃圾邮件头…
mail-spam-score-unavailable = 垃圾邮件评分不可用
mail-spam-score-unverified = 由邮件头报告，未经独立验证。发件人提供的邮件头可能被伪造。
mail-spam-score-positive = 邮件头报告为垃圾邮件。
mail-spam-score-negative = 邮件头报告为非垃圾邮件。
mail-spam-score-unknown = 此评分未附带结论。
mail-external-recipients = 外部收件人
mail-external-count = { $count } 位收件人不在受信任域名内
mail-external-help = 发送机密信息前，请检查收件人、抄送和密送的收件人。信任范围取决于所选的发送身份和你的有效域名策略。
mail-external-reviewed = 我已检查这些外部收件人。
mail-external-confirm-required = 发送前请检查并确认外部收件人。
mail-quick-reply = 快速回复
mail-quick-reply-help = 提供已填好收件人的紧凑回复标签页。默认关闭。常规安全检查和发送选项仍然适用。
mail-quick-reply-expand = 编辑收件人和主题
mail-attachment-policy-loading = 正在检查附件策略…
mail-attachment-policy-unavailable = 无法检查附件策略。请在打开此文件前重试。
mail-attachment-policy-blocked = 管理员已阻止此附件或其原始文件下载。
mail-attachment-policy-warning = 管理员建议谨慎处理此附件。仅在信任其来源时才继续。
mail-attachment-policy-acknowledge = 我了解——继续
mail-attachment-policy-no-preview = 管理员已禁用附件预览。可能仍可下载原始文件。
mail-time-sent = 发送时间
mail-time-received = 接收时间
mail-time-unknown = 未提供
mail-time-comparison = { $zone } 时间
mail-time-privacy = 对比时区由你选择。邮件时间戳不会标识任何人的位置。
mail-time-event-comparison = 活动时间对比
mail-time-ambiguous = 时钟调整时，此本地时间会出现两次。两个可能的时刻均已显示；保存前请确认活动预期的偏移量。
mail-bulk-add-label = 添加标签
mail-select-messages = 选择邮件
mail-bulk-remove-label = 移除标签
mail-mark-spam = 标记为垃圾邮件
mail-bulk-forward = 将所选邮件作为附件转发
mail-bulk-download = 将所选邮件下载为 mbox
mail-bulk-forward-subject = 转发的邮件（{ $count }）
mail-bulk-transfer-failed = 无法准备所选邮件。请选择最多 100 封、总计不超过 32 MiB 的邮件，并检查账户的导出权限。
mail-toast-labels-added = 已为 { $count } 封邮件添加标签 { $label }
mail-toast-labels-removed = 已从 { $count } 封邮件移除标签 { $label }
mail-action-help-reply = 在新的撰写标签页中回复发件人。
mail-action-help-reply-all = 在新的撰写标签页中回复发件人和其他收件人。
mail-action-help-forward = 查看此邮件并转发给你选择的收件人。
mail-action-help-pin = 固定此邮件以便快速访问，或取消固定。
mail-action-help-read = 更改此邮件的已读标记。你可以撤销此更改。
mail-action-help-flag = 设置或取消邮件的跟进标记。
mail-action-help-archive = 将此邮件移至已配置的归档文件夹。
mail-action-help-delete = 将此邮件移至废纸篓文件夹。你可以撤销此移动。
mail-action-help-spam = 将此邮件标记为垃圾邮件并移至已配置的垃圾邮件文件夹。
mail-action-help-export = 打开导出页面，以所选格式下载此邮件。
mail-action-help-sweep = 先查看该发件人的其他邮件，并选择清理方式（全部删除、保留最新一封、删除较早的邮件或屏蔽发件人），然后再应用。
mail-empty-trash = 清空废纸篓
mail-trash-help = 查看并永久删除当前废纸篓中的邮件。此操作无法撤销。
mail-trash-preparing = 正在准备废纸篓中邮件的快照…
mail-trash-prepare-failed = 无法准备废纸篓预览。请检查网络连接后重新打开。未删除任何邮件。
mail-trash-confirm = 永久删除废纸篓中的 { $count } 封邮件？此操作无法撤销。此预览之后到达的邮件不包括在内；已移出废纸篓的邮件将被跳过。
mail-trash-limit = 此操作最多包含 { $count } 封邮件。废纸篓中仍有更多邮件；请之后另行发起操作将其删除。
mail-trash-permanent = 永久删除
mail-trash-stop = 在当前请求完成后停止
mail-trash-progress = 已删除 { $deleted } 封；已跳过 { $skipped } 封不在废纸篓中的邮件；此预览中还剩 { $remaining } 封。
mail-trash-stopped = 删除已停止。确认已删除：{ $deleted }；已跳过：{ $skipped }；剩余：{ $remaining }；结果未知：{ $uncertain }。开始另一项操作前请检查废纸篓。
mail-links-title = 邮件链接
mail-links-review = 检查链接（{ $count }）
mail-links-help = 只有在此处检查过目标地址后，邮件链接才会打开。不会自动获取任何目标地址。
mail-links-inert = 请使用邮件上方的“检查链接”来查看并打开此目标地址。
mail-links-limit = 仅列出前 100 个不重复的受支持目标地址。其余链接在邮件中保持未激活状态。
mail-links-destination = 要打开的目标地址
mail-links-external = 此目标地址来自邮件发件人。打开后将离开 Mailwoman；继续前请检查地址。
mail-links-open = 在新标签页中打开
mail-links-compose = 检查新邮件
mail-links-warning-http = 此地址使用未加密的 HTTP。
mail-links-warning-address = 此目标地址使用 IP 地址而不是域名。
mail-links-warning-international = 此域名包含国际化名称。请仔细检查其 ASCII 拼写。
mail-links-warning-mismatch = 显示的网址与实际目标地址指向不同的主机。
mail-links-warning-direction = 链接文本包含可能伪装其地址的方向变换字符。
mail-outbox-uncertain = 投递状态未知
mail-outbox-uncertain-note = 已开始发送，但无法确认是否送达。此邮件不会自动重新发送。再次发送前，请检查已发送邮件或向收件人确认。
mail-conversation-title = 查看会话
mail-conversation-help = 浏览此账户跨文件夹的可用会话邮件。打开邮件不会更改你选择的文件夹。
mail-conversation-count = 显示会话邮件 { $loaded } / { $total }。
mail-conversation-other-folder = 其他文件夹
mail-conversation-loading = 正在加载会话…
mail-conversation-missing = 有 { $count } 封邮件已不可用。
mail-conversation-error = 无法加载会话。请重试。
mail-conversation-more = 加载更多邮件
mail-conversation-limit = 此视图最多显示 { $count } 封邮件。
mail-compose-request-delivery = 请求投递通知
mail-compose-request-delivery-hint = 请接收方邮件服务器报告投递、失败或延迟。这不会请求或证明任何人已阅读邮件。返回的报告包含邮件头，不含完整正文。
mail-compose-delivery-unavailable = 管理员已为此账户停用或不提供投递通知。明确请求的通知不会被悄悄移除。
mail-compose-account-changed = 此撰写内容属于另一个登录账户。请登录正确的账户后重新打开草稿。
mail-context-open-message = 打开邮件
mail-context-open-folder = 打开文件夹
mail-context-download = 下载邮件
mail-context-refresh = 刷新文件夹
mail-context-folder-settings = 文件夹设置
mail-context-failed = 无法完成该操作。请重试。
mail-sharing-unavailable = 文件夹共享不可用，或你已无权管理它。
mail-trash-cancelling = 将在当前请求完成后停止。确认已删除：{ $deleted }；已跳过：{ $skipped }；剩余：{ $remaining }。已接受的删除无法撤销。
mail-trash-already-running = 废纸篓删除正在进行。请使用其进度通知来停止后续批次。
mail-context-search-sender = 搜索此发件人
mail-context-search-subject = 搜索相同主题
mail-drag-too-many = 每次最多可拖动 { $count } 封邮件。请在移动前选择较少的邮件。
mail-subtab-reorder-help = 拖动以重新排列标签页，或在活动标签页上按 Ctrl+Shift+左箭头或右箭头。
mail-compose-ribbon = 编辑器功能区
mail-compose-ribbon-format = 设置文本格式
mail-compose-ribbon-insert = 插入
mail-compose-font = 字体
mail-compose-font-size = 字号
mail-compose-font-inherit = 默认
mail-compose-undo = 撤销
mail-compose-redo = 重做
mail-compose-undo-tip = 撤销（{ $shortcut }）
mail-compose-redo-tip = 重做（{ $shortcut }）
mail-workspace-limit = 请先关闭一个标签页再打开另一个。最多可保留 32 个标签页。
mail-compose-private-storage-unavailable = 此会话无法使用本地草稿恢复。在发送或复制邮件之前，请保持此编辑器打开。
mail-offline-storage-unavailable = 在此会话验证存储权限之前，离线更改不可用。请重新连接并再次登录。
mail-draft-status-idle = 尚未保存到账户
mail-draft-status-pending = 有未保存的更改
mail-draft-status-saving = 正在保存到草稿…
mail-draft-status-saved = 已保存到账户草稿
mail-draft-status-error = 草稿未保存。你的本地恢复副本仍然可用。
mail-draft-status-uncertain = 缺少保存确认。创建另一个副本前请检查草稿；自动保存已暂停。
mail-draft-status-conflict = 此草稿已在另一台设备上更改。继续前请重新打开其账户副本。
mail-draft-status-unsupported = 此账户目前无法确认对其草稿文件夹的保存。
mail-draft-protected = 加密或已解密的引用内容不会以明文保存。之前的账户草稿会一直保留，直到你明确丢弃。
mail-draft-save-now = 保存到草稿
mail-draft-save-failed = 无法将此草稿保存到你的账户。关闭前请检查撰写窗口。
mail-draft-uncertain = 草稿可能已保存。自动重试已暂停；请检查草稿文件夹。
mail-draft-conflict = 另一台设备更改了此草稿。你的编辑未覆盖其副本。
mail-draft-cleanup-pending = 新草稿已保存，但无法删除其之前的副本。再次保存前请检查草稿。
mail-draft-account-list = 账户草稿
mail-draft-loading = 正在加载账户草稿…
mail-draft-close-recovery = 关闭并仅保留本地恢复
mail-draft-discard-account = 丢弃
mail-draft-discard-pending = 无法确认草稿已删除。其账户记录仍然存在；检查草稿文件夹后请重试“丢弃”。
mail-draft-limit = 此账户已有 100 份已保存的草稿。保存新草稿前，请丢弃不需要的账户草稿。
mail-draft-quota-help = 最多保留 100 份账户草稿。丢弃时会先删除已确认的上游副本，再释放其名额。
mail-draft-discard-confirm = 确认删除草稿
mail-draft-discard-confirm-detail = 从账户草稿中永久删除“{ $subject }”？若仅关闭撰写窗口，则会保留该草稿。
mail-draft-discard-title = 丢弃此草稿？
mail-draft-keep-editing = 继续编辑
mail-draft-discard-action = 丢弃
mail-draft-close-title = 丢弃未保存的更改？
mail-draft-close-protected = 此邮件包含加密或已解密的内容，因此不会保存为草稿。关闭将丢弃这些更改。之前的账户草稿（如有）会保留在草稿中。
mail-draft-close-unstored = 此会话无法保存草稿，因此关闭此邮件将丢弃它。
mail-workspace-view-mode = 视图排列
mail-workspace-tabs-help = 显示一个活动视图，同时保持其他标签页打开。
mail-workspace-side-by-side-help = 并排显示两个所选视图。将标签页拖到任一视图上可替换它。
mail-workspace-stack-help = 上下显示两个所选视图。将标签页拖到任一视图上可替换它。
mail-workspace-masonry-side-by-side = 瀑布流并排
mail-workspace-masonry-side-by-side-help = 将所有打开的视图以交错列平铺。小屏幕显示活动标签页。
mail-workspace-masonry-stack = 瀑布流堆叠
mail-workspace-masonry-stack-help = 将所有打开的视图以交错行平铺。小屏幕显示活动标签页。
mail-workspace-view-first = 第一视图
mail-workspace-view-second = 第二视图
mail-workspace-view-empty = 将已打开的标签页拖到此处以选择此视图。
mail-popup-blocked = 请允许 Mailwoman 弹出窗口以分离此标签页。你的标签页仍保留在此。
mail-popup-unavailable = 无法加载分离的窗口。你的标签页和未保存的更改仍保留在主窗口中。请刷新应用后重试。
mail-popup-reattach = 重新附加到主窗口
mail-popup-action-reattached = 已返回主窗口以完成此操作。
mail-popup-detached = 此标签页在单独的窗口中打开。关闭该窗口即可返回此处。
mail-compose-from-custom = 其他已授权地址…
mail-compose-from-alias = 此邮件的发件地址
mail-compose-from-alias-help = 使用你有权发送的别名。这仅更改此草稿；发送前会检查授权。
mail-compose-from-refused = 此地址未被授权通过此账户发送。
mail-compose-from-unavailable = 无法验证此发件地址。你的邮件尚未发送。
mail-compose-from-invalid = 请为发件人输入一个有效的电子邮件地址。
mail-compose-from-toggle = 显示或隐藏此邮件的发件人
mail-compose-ribbon-security = 安全
mail-compose-ribbon-delivery = 投递
mail-compose-ribbon-recipients = 收件人
mail-compose-ribbon-message-type = 邮件类型
mail-compose-recall-help = 取消仍在撤销等待或定时发送中的邮件。已投递的邮件无法召回。
mail-compose-recall-loading = 正在检查排队中的邮件…
mail-compose-recall-empty = 没有可取消的排队邮件。

# -- Machine translation (2026-11): added to match en ---------------------
mail-compose-recall-failed = 无法确认队列状态。请关闭并重新打开此面板以刷新，然后再执行其他操作。
mail-compose-importance = 重要性
mail-compose-importance-high = 高
mail-compose-importance-normal = 普通
mail-compose-importance-low = 低
mail-insert-account-title = 插入账户项目
mail-insert-account-help = 附加此账户中的日历导出文件、联系人名片或原始邮件。
mail-insert-account-close = 关闭账户项目选择器
mail-insert-account-kind = 项目类型
mail-insert-account-event = 日历活动 (.ics)
mail-insert-account-calendar = 日历 (.ics)
mail-insert-account-contact = 联系人名片 (.vcf)
mail-insert-account-message = 原始邮件 (.eml)
mail-insert-account-search = 筛选列出的项目
mail-insert-account-selection = 选择一个项目
mail-insert-account-scope = 仅列出当前账户中的项目。原始邮件遵循服务器的导出策略。
mail-insert-account-limited = 最多显示 100 个项目。筛选仅搜索此列表。
mail-insert-account-unavailable = 没有可用的受支持账户导出。
mail-insert-account-empty = 没有匹配的项目。
mail-insert-account-attach = 附加所选项目
mail-insert-account-loading = 正在加载账户项目…
mail-insert-account-error = 无法附加该项目。请检查你的访问权限和导出策略后重试。较大的日历可能需要逐个导出活动。
mail-compose-html-source = HTML 源码
mail-editor-clear-format = 清除格式
mail-compose-format-html = HTML
mail-compose-html-visual = 可视化编辑器
mail-compose-html-help = 直接编辑 HTML。仅保存和发送受支持的邮件格式；脚本、外部图片和不安全的标记会被移除。
mail-compose-html-processing = 正在准备安全的 HTML…
mail-compose-html-failed = 无法准备 HTML。请先编辑或重试，再保存或发送。
mail-compose-check-names = 检查姓名
mail-compose-check-names-help = 根据已加载的联系人解析完全匹配的姓名，并检查地址格式。这不会验证邮箱能否投递。
mail-compose-names-unresolved = 请为这些未解析或有歧义的姓名选择完整的电子邮件地址：{ $names }
mail-compose-names-checked = 已检查收件人格式；已解析 { $count } 个联系人姓名。尚未验证邮箱能否投递。
mail-compose-ribbon-table = 表格
mail-compose-group-clipboard = 剪贴板
mail-compose-group-paragraph = 段落
mail-compose-group-editing = 编辑
mail-compose-group-view = 视图
mail-compose-dark-background = 深色背景
mail-compose-dark-background-help = 撰写时在深色背景上显示邮件。仅改变你的屏幕显示：发送的邮件不带背景色。
mail-compose-group-assist = 写作助手
mail-editor-counts = 编辑器统计
mail-editor-words = { $count ->
   *[other] { $count } 个字词
}
mail-editor-paragraphs = { $count ->
   *[other] { $count } 个段落
}
mail-editor-characters = { $count ->
   *[other] { $count } 个字符
}
mail-editor-selection-required = 使用此操作前，请先选择合适的文本或表格单元格。
mail-editor-clipboard-unavailable = 无法访问剪贴板。请使用浏览器的复制、剪切或粘贴快捷键。
mail-editor-selection-changed = 所选内容已更改。请选择目标文本后重试。
mail-editor-clipboard = 剪贴板和格式
mail-editor-copy = 复制
mail-editor-cut = 剪切
mail-editor-paste = 粘贴
mail-editor-copy-format = 复制格式
mail-editor-apply-format = 应用格式
mail-editor-style = 段落样式
mail-editor-text-color = 字体颜色
mail-editor-clear-color = 默认颜色
mail-editor-highlight = 突出显示
mail-editor-clear-highlight = 清除突出显示
mail-editor-paragraph = 段落选项
mail-editor-alignment = 对齐方式
mail-editor-line-height = 行高
mail-editor-space-before = 段前间距
mail-editor-space-after = 段后间距
mail-editor-indent = 缩进
mail-editor-shading = 段落底纹
mail-editor-clear-shading = 清除底纹
mail-editor-case = 更改大小写
mail-editor-sort = 对所选段落排序
mail-editor-sort-alpha = 按字母顺序
mail-editor-sort-numeric = 按数字顺序
mail-editor-table-rows = 表格行数
mail-editor-table-columns = 表格列数
mail-editor-table-insert = 插入表格
mail-editor-table-limits = 每个表格最多 20 行、10 列和 200 个单元格；最多 10 个表格。不支持嵌套表格。
mail-editor-table-row-before = 在上方插入行
mail-editor-table-row-after = 在下方插入行
mail-editor-table-row-delete = 删除行
mail-editor-table-column-before = 在左侧插入列
mail-editor-table-column-after = 在右侧插入列
mail-editor-table-column-delete = 删除列
mail-editor-table-merge = 合并单元格
mail-editor-table-split = 拆分单元格
mail-editor-table-header = 切换标题行
mail-editor-table-delete = 删除表格
mail-editor-table-shading = 单元格底纹
mail-editor-find-replace = 查找和替换
mail-editor-find = 查找文本
mail-editor-replacement = 替换为
mail-editor-match-case = 区分大小写
mail-editor-previous = 上一个匹配项
mail-editor-next = 下一个匹配项
mail-editor-replace = 替换所选匹配项
mail-editor-replace-all = 替换所有匹配项
mail-editor-matches = { $count } 个匹配项（最多 1,000 个）
mail-editor-select-all = 全选
mail-editor-clear-selection = 清除选择
mail-editor-history = 撤销历史
mail-editor-undo-groups = 撤销最近 { $count } 组更改
mail-editor-zoom = 编辑器缩放
mail-editor-markers = 段落标记
mail-editor-style-normal = 正文
mail-editor-style-no-spacing = 无间距
mail-editor-style-title = 标题
mail-editor-style-subtitle = 副标题
mail-editor-style-heading1 = 标题 1
mail-editor-style-heading2 = 标题 2
mail-editor-style-heading3 = 标题 3
mail-editor-style-heading4 = 标题 4
mail-editor-style-heading5 = 标题 5
mail-editor-style-heading6 = 标题 6
mail-editor-align-left = 左对齐
mail-editor-align-center = 居中
mail-editor-align-right = 右对齐
mail-editor-align-justify = 两端对齐
mail-editor-case-lower = 小写
mail-editor-case-upper = 大写
mail-editor-case-title = 每个单词首字母大写
mail-editor-case-sentence = 句首字母大写
mail-editor-action-history = { $action }历史
mail-editor-history-empty = 没有可用的操作
mail-editor-redo-groups = 重做 { $count } 项更改
mail-compose-identities-failed = 无法加载发送身份。请关闭并重新打开此草稿后重试。
mail-folders-loading = 正在加载文件夹…
mail-initial-sync-error = 暂时无法同步你的邮件。
mail-initial-sync-error-help = 与邮件服务器的连接已中断。请重试；如果持续出现，请联系管理员检查连接日志。
mail-links-trusted = 受信任域名：无需检查即可打开
mail-links-trust-domain = 信任 { $domain }
mail-links-trust-failed = 无法保存受信任域名。请重试。
mail-focus-always-focused = 始终移至“重点”
mail-focus-always-other = 始终移至“其他”
mail-focus-moved-focused = 来自 { $sender } 的邮件现在会进入“重点”。
mail-focus-moved-other = 来自 { $sender } 的邮件现在会进入“其他”。
mail-focus-save-failed = 无法保存此发件人的“重点”收件箱选择。请重试。
mail-context-search-domain = 搜索来自此域名的邮件
mail-loading-mailbox = 正在加载 { $name }…
mail-user-folders = 用户文件夹
mail-context-hide-folder = 从侧边栏隐藏
mail-folder-fallback = 文件夹
mail-toast-moving = { $count ->
    [one] 正在将 1 封邮件移至 { $folder }…
   *[other] 正在将 { $count } 封邮件移至 { $folder }…
    }
mail-toast-move-queueing = { $count ->
    [one] 正在将 1 封邮件加入移至 { $folder } 的队列…
   *[other] 正在将 { $count } 封邮件加入移至 { $folder } 的队列…
    }
mail-toast-move-queued = { $count ->
    [one] 已将 1 封邮件加入移至 { $folder } 的队列
   *[other] 已将 { $count } 封邮件加入移至 { $folder } 的队列
    }
mail-toast-move-partial = 已将 { $total } 封邮件中的 { $count } 封移至 { $folder }；有 { $failed } 封无法移动
mail-send-copy-folder-unavailable = 所选的已发送副本文件夹不可用。发送前请选择其他文件夹。
mail-send-copy-failed = 邮件已被接受，但无法归档其已发送副本。请勿再次发送。
mail-compose-attach-public-key = 附加我的公钥或证书
mail-compose-include-hashes = 包含所附文件的 SHA-256 哈希值
mail-compose-hashes-pending = 正在计算附件哈希值。准备好后再发送。
mail-compose-hashes-failed = 无法验证附件字节。请关闭哈希值，或重新启用后重试。
mail-compose-crypto-pending = 加密正在更新。请等待后再发送。
mail-compose-public-key-unavailable = 没有与发件地址匹配的当前公钥或证书。
mail-compose-sent-copy = 将已发送邮件保存到
mail-compose-sent-copy-default = 账户默认
folder-create = 新建文件夹
folder-name = 文件夹名称
folder-parent = 上级文件夹
folder-top-level = 顶层
folder-created = 文件夹已创建。
folder-create-unavailable = 此账户或上级文件夹无法创建文件夹。
folder-create-invalid = 请选择有效且唯一的文件夹名称，不含层级分隔符或控制字符。
folder-create-failed = 无法创建文件夹。请刷新后重试。
folder-cleanup = 清理文件夹…
folder-cleanup-title = 清理 { $name }
folder-cleanup-help = 查找早于所选时间的邮件。删除前请先核对数量。子文件夹不包括在内。
folder-cleanup-age = 早于
folder-age-days = 天
folder-age-weeks = 周
folder-age-months = 个月
folder-age-years = 年
folder-cleanup-preview = 预览清理
folder-cleanup-confirm = 删除匹配的邮件
folder-cleanup-permanent = 永久删除 { $count } 封匹配的邮件。此操作无法撤销。
folder-cleanup-trash = 将 { $count } 封匹配的邮件移至废纸篓。
folder-cleanup-before = 接收时间早于 { $date }
folder-cleanup-done = 已清理 { $count } 封邮件。
mail-from = 发件人
mail-subject = 主题
mail-date = 日期
mail-cancel = 取消
mail-bulk-selected = 已选择 { $count } 个会话
mail-bulk-select-folder = 全选 { $folder } 中的邮件
mail-list-order = 邮件排序

# -- Machine translation (2026-11): added to match en ---------------------
mail-list-order-newest = 最新的在前
mail-list-order-oldest = 最早的在前
mail-list-order-sender = 发件人
mail-list-order-subject = 主题
mail-list-order-error = 无法保存邮件排序。
mail-bulk-selection-load-failed = 无法选择整个文件夹。请检查网络连接后重试。
mail-privacy-bar = 邮件隐私
mail-privacy-trackers-allow = 允许此邮件使用跟踪器
mail-privacy-trackers-block = 再次阻止跟踪器
mail-privacy-images-loading = 已显示 { $loaded }/{ $total } 张图片 · { $pending } 张加载中
mail-privacy-images-failed = 已显示 { $loaded }/{ $total } 张图片 · { $failed } 张失败
mail-privacy-images-loaded = 已显示 { $loaded }/{ $total } 张图片
mail-privacy-images-loading-failed = 已显示 { $loaded }/{ $total } 张图片 · { $pending } 张加载中 · { $failed } 张失败
mail-reader-folder-title = 文件夹：{ $folder }
mail-reader-move-to = 移至…
mail-action-help-move = 将此邮件移至另一个文件夹。
folder-ops-new-subfolder = 新建子文件夹
folder-ops-mark-read = 全部标记为已读
folder-ops-this-folder = 此文件夹
folder-ops-with-subfolders = 包含子文件夹
folder-ops-no-unread = 没有未读邮件。
folder-ops-read-unavailable = 你无法更改此文件夹中的邮件。
folder-ops-more = 更多
folder-ops-system-folder = 系统文件夹无法通过此方式更改。
folder-ops-sort-subfolders = 子文件夹排序方式
folder-ops-sort-folders = 文件夹排序方式
folder-ops-sort-name = 名称
folder-ops-sort-unread = 未读数量
folder-ops-sort-total = 总数量
folder-ops-sort-date = 最近活动
folder-ops-sort-custom = 自定义顺序
folder-ops-sort-asc = 升序
folder-ops-sort-desc = 降序
folder-ops-delete-with-subfolders = 删除文件夹及子文件夹
folder-ops-delete-unavailable = 无法删除此文件夹。
folder-ops-delete-tree-body = 这将永久删除 { $folders } 个文件夹及其中的 { $count } 封邮件。此操作无法撤销。
folder-ops-copy-all-menu = 将所有项目复制到…
folder-ops-move-all-menu = 将所有项目移至…
folder-ops-move-folder-menu = 将文件夹移至…
folder-ops-move-up = 上移
folder-ops-move-down = 下移
folder-ops-move-unavailable = 你无法移动此文件夹或其中的项目。
folder-ops-move-duplicate = 目标位置已有同名文件夹。
folder-ops-move-folder-title = 移动 { $name }
folder-ops-move-folder-body = 文件夹将连同其子文件夹和邮件一起移动。请选择要放入的文件夹，或选择顶层。
folder-ops-move-here = 移至此处
folder-ops-moved-folder = 已移动 { $name }。
folder-ops-choose-folder = 请选择目标文件夹。
folder-ops-destination = 目标文件夹
folder-ops-include-subfolders = 包含 { $count } 个子文件夹中的邮件
folder-ops-copy-all-title = 复制 { $name } 的所有项目
folder-ops-move-all-title = 移动 { $name } 的所有项目
folder-ops-copy-all-body = 将 { $count } 封邮件添加到目标文件夹。原邮件保留在原处。
folder-ops-move-all-body = 将 { $count } 封邮件移至目标文件夹。之后你可以撤销此操作。
folder-ops-copy-all = 复制项目
folder-ops-move-all = 移动项目
folder-ops-undo = 撤销
folder-ops-stop = 停止
folder-ops-marking = 正在将 { $name } 标记为已读…
folder-ops-marking-progress = 正在将 { $name } 标记为已读：{ $done } / { $total }
folder-ops-marked = 已将 { $name } 中的 { $count } 封邮件标记为已读。
folder-ops-copying = 正在复制 { $name }…
folder-ops-copying-progress = 正在复制 { $name }：{ $done } / { $total }
folder-ops-copied = 已从 { $name } 复制 { $count } 封邮件。
folder-ops-moving = 正在移动 { $name }…
folder-ops-moving-progress = 正在移动 { $name }：{ $done } / { $total }
folder-ops-moved = 已从 { $name } 移动 { $count } 封邮件。
folder-ops-stopped = 已在处理 { $total } 封中的 { $count } 封邮件后停止 { $name }。
folder-ops-partial = { $name } 中 { $total } 封邮件已处理 { $count } 封；{ $refused } 封被拒绝。
folder-ops-failed = 无法完成 { $name }。请刷新后重试。
folder-ops-error-forbidden = 你没有执行此更改的权限。
folder-ops-error-exists = 那里已存在同名文件夹。
folder-ops-error-mailboxHasEmail = 该文件夹中仍有邮件。
folder-ops-error-notFound = 该文件夹已不存在。
folder-ops-error-failed = 无法进行此更改。请刷新后重试。
folder-ops-expand = 显示 { $name } 的子文件夹
folder-ops-collapse = 隐藏 { $name } 的子文件夹
mail-context-reply = 回复
mail-context-reply-all = 回复全部
mail-context-forward = 转发
mail-context-quick-print = 快速打印
mail-context-copy = 复制
mail-context-copy-to-folder = 将邮件复制到文件夹…
mail-context-copy-sender = 复制发件人地址
mail-context-copy-subject = 复制主题
mail-context-copy-link = 复制邮件链接
mail-context-copied = 已复制。
mail-context-copy-failed = 无法复制到剪贴板。
mail-context-print-failed = 无法打印此邮件。
mail-reply-images-title = 原邮件中的图片
mail-reply-images-ask = 在此回复中包含原邮件的图片吗？
mail-reply-images-include = 包含图片
mail-reply-images-not-now = 暂不
mail-reply-images-trust = 始终信任 { $domain }
mail-reply-images-loading = 正在添加图片…
mail-reply-images-cancelled = 加载已取消。未添加图片。
mail-reply-images-failed = 无法添加图片。
mail-reply-images-retry = 重新加载
mail-image-cancel = 取消
mail-compose-from-why-match = 使用 { $identity } 发送，因为该邮件是发送到 { $address } 的。
mail-compose-from-why-match-plus = 使用 { $identity } 发送，因为该邮件是发送到 { $address } 的加号地址。
mail-compose-from-why-ambiguous = 有多个身份与 { $address } 匹配。将使用 { $identity } 发送；可另选其他身份：
mail-compose-from-why-default = 使用你的默认身份 { $identity } 发送，因为没有身份与原收件人匹配。
mail-compose-from-why-forced-default = 按你的设置，使用默认身份 { $identity } 发送。
mail-compose-from-why-keep = 使用 { $identity } 发送。
mail-compose-from-change = 更改
reader-attachments-reason-encrypted = 此邮件已加密。解锁后才会显示其附件。
reader-attachments-reason-list-missing = 无法加载此邮件的附件列表，因此已隐藏附件栏。
reader-attachments-reason-inline-only = 此邮件的附件（{ $count }）是显示在邮件正文中的内嵌图片。
reader-attachments-reason-unavailable-parts = { $count ->
   *[other] 此邮件有 { $count } 个附件没有可下载的内容，因此已隐藏附件栏。
  }
reader-attachments-reason-policy-blocked = 你的附件策略禁止下载{ $count ->
   *[other] 这些附件。
  }
reader-attachments-reason-policy-unavailable = 无法检查附件策略，因此暂时无法打开附件。
mail-drafts-popup-count = { $count ->
   *[other] { $count } 份草稿
  }
mail-drafts-popup-search = 按主题或收件人搜索
mail-drafts-popup-tab-local = 本地草稿
mail-drafts-popup-local-hint = 本地草稿仅保存在此浏览器中。
mail-drafts-popup-badge-account = 账户
mail-drafts-popup-badge-local = 本地
mail-drafts-popup-current = 当前已打开
mail-drafts-popup-select = 选择
mail-drafts-popup-select-all = 全选
mail-drafts-popup-clear = 清除
mail-drafts-popup-done = 完成
mail-drafts-popup-selected = 已选择 { $count } 项
mail-drafts-popup-select-row = 选择 { $subject }
mail-drafts-popup-open = 打开
mail-drafts-popup-discard = 丢弃
mail-drafts-popup-discard-selected = 丢弃所选
mail-drafts-popup-more = { $subject } 的更多操作
mail-drafts-popup-retry = 重试
mail-drafts-popup-load-failed = 无法加载账户草稿。
mail-drafts-popup-empty-account = 没有账户草稿
mail-drafts-popup-no-match = 没有与“{ $query }”匹配的草稿
mail-drafts-popup-discarding = 正在丢弃…
mail-drafts-popup-row-failed = 无法丢弃。其账户副本仍然保留。
mail-drafts-popup-confirm-title = { $count ->
   *[other] 丢弃 { $count } 份草稿？
  }
mail-drafts-popup-confirm-account = 账户草稿会先从你账户的草稿文件夹中删除，只有在确认删除后才会释放其名额。
mail-drafts-popup-confirm-local = 本地草稿将从此浏览器中删除。
mail-drafts-popup-announce-done = { $count ->
   *[other] 已丢弃 { $count } 份草稿
  }
mail-drafts-popup-announce-failed = 已丢弃 { $done } 份，{ $failed } 份无法丢弃
mail-check-title = 发送前检查
mail-check-intro = 以下内容可能需要再看一眼：
mail-check-invalid = 部分收件人地址似乎无效：{ $list }
mail-check-subject = 主题为空。
mail-check-attachment = 邮件中提到了附件，但没有附加任何文件。
mail-check-external = 不在受信任域名内的收件人：{ $list }
mail-check-send-anyway = 仍然发送
mail-check-review = 检查
mail-check-remember = 对这些不再询问
mail-check-denied-invalid = 你的设置禁止发送。请修正无效的收件人地址：{ $list }
mail-check-denied-subject = 你的设置禁止发送。请添加主题。
mail-check-denied-attachment = 你的设置禁止发送。请附加你提到的文件，或修改邮件措辞。
mail-check-denied-external = 你的设置禁止发送。不在受信任域名内：{ $list }
mail-alerts-label = 邮件问题
mail-alert-invalid = 无效地址：{ $address }
mail-alert-subject = 无主题
mail-recipient-chip-remove = 移除 { $address }
mail-recipient-chips = { $field } 中的收件人
mail-recipient-invalid-title = 不是有效的电子邮件地址
mail-recipient-chip-name = { $name }，{ $address }。按 Enter 查看联系人详情，按 Backspace 或 F2 编辑，按 Delete 移除
mail-recipient-chip-name-invalid = { $address }，不是有效地址。按 Enter 编辑，按 Delete 移除
mail-recipient-added = 已添加 { $name }
mail-recipient-removed = 已移除 { $name }
mail-recipient-removed-many = { $count ->
   *[other] 已移除 { $count } 位收件人
}
mail-recipient-editing = 正在编辑 { $name }
mail-recipient-all-selected = { $count ->
   *[other] 已选择 { $count } 位收件人
}
mail-external-line = { $count } 位不在受信任域名内：{ $domains }
mail-external-dismiss = 关闭
mail-status-label = 撰写状态

# -- Machine translation (2026-11): added to match en ---------------------
mail-status-more = 另有 { $count } 条提示
mail-status-more-label = 更多提示
mail-status-no-recipients = 没有收件人
mail-status-no-attachment = 没有附件
mail-status-external = 外部：{ $count }
mail-status-external-reviewed = 已检查外部收件人
mail-status-external-review = 标记为已检查
mail-status-from-match = 发件人：{ $identity }
mail-status-from-match-plus = 发件人：{ $identity }
mail-status-from-ambiguous = 发件人：{ $identity }（有多个匹配）
mail-status-from-default = 发件人：默认身份
mail-status-from-forced-default = 发件人：默认身份
mail-status-from-keep = 发件人：{ $identity }
mail-status-held = { $count } 封排队中
mail-status-storage = 无本地恢复
mail-status-protected = 不以明文保存
mail-status-saving = 正在保存…
mail-status-saved = 已保存 { $time }
mail-status-saved-full = 已保存到账户草稿 · { $time }
mail-status-not-saved = 草稿未保存
mail-status-retry = 重试
mail-status-conflict = 已在另一台设备上更改
mail-status-uncertain = 保存未确认
mail-status-close-recovery = 关闭并保留恢复
mail-status-unsupported = 草稿保存未确认
mail-compose-header = 邮件头部
mail-header-label = 邮件头部
mail-header-collapse = 折叠邮件头部
mail-header-expand = 展开邮件头部：{ $summary }
mail-header-from-chip = 发件人 { $address }。更改发件人
mail-compose-group-timing = 发送时间
mail-compose-group-priority = 优先级
mail-compose-group-receipts = 回执和跟踪
mail-compose-group-keys-files = 密钥和文件
mail-compose-group-sent-copy = 已发送副本
mail-editor-style-quote = 引用
mail-editor-style-code = 代码
mail-compose-ribbon-table-layout = 表格布局
mail-compose-ribbon-picture = 图片
mail-editor-table-group-rows = 行
mail-editor-table-group-columns = 列
mail-editor-table-group-cells = 单元格
mail-editor-table-row-height = 行高
mail-editor-table-column-width = 列宽
mail-editor-table-size-auto = 自动
mail-editor-table-padding = 单元格边距
mail-editor-table-padding-compact = 紧凑
mail-editor-table-padding-normal = 标准
mail-editor-table-padding-roomy = 宽松
mail-editor-table-align-left = 单元格左对齐
mail-editor-table-align-center = 单元格居中
mail-editor-table-align-right = 单元格右对齐
mail-picture-group-size = 大小
mail-picture-group-arrange = 排列
mail-picture-group-text = 文字
mail-picture-group-style = 样式
mail-picture-group-edit = 编辑
mail-picture-width = 宽度
mail-picture-height = 高度
mail-picture-lock = 保持比例
mail-picture-reset-size = 原始大小
mail-picture-align-inline = 与文字同行
mail-picture-align-left = 左对齐
mail-picture-align-center = 居中
mail-picture-align-right = 右对齐
mail-picture-alt = 替代文字
mail-picture-title = 标题
mail-picture-border = 边框
mail-picture-radius = 圆角
mail-picture-none = 无
mail-picture-round = 圆形
mail-picture-rotate-ccw = 向左旋转
mail-picture-rotate-cw = 向右旋转
mail-picture-flip-h = 水平翻转
mail-picture-flip-v = 垂直翻转
mail-picture-crop = 裁剪
mail-picture-crop-left = 左（%）
mail-picture-crop-top = 上（%）
mail-picture-crop-right = 右（%）
mail-picture-crop-bottom = 下（%）
mail-picture-crop-apply = 裁剪
mail-picture-replace = 替换
mail-picture-remove = 移除
mail-picture-failed-no-image = 请先选择一张图片。
mail-picture-failed-no-source = 此图片没有可编辑的本地副本。请改为替换它。
mail-picture-failed-failed = 无法更改图片。
mail-context-download-attachments = 下载附件
mail-context-download-named = 下载 { $name }
mail-context-download-all = 全部下载为 ZIP
mail-context-loading-attachments = 正在加载附件…
mail-context-attachments-failed = 无法加载附件。
mail-context-searching = 正在查找邮件…
mail-context-go-reply = 转到回复
mail-context-go-forward = 转到转发
mail-context-reply-not-found = 未找到回复
mail-context-forward-not-found = 未找到转发
mail-no-recipients = （无收件人）
mail-attachment-downloaded = 已下载 { $name }
mail-attachment-download-failed = 下载失败。
mail-attachments-zip-done = 已将 { $count } 个附件下载为 ZIP。
mail-attachments-zip-capped = ZIP 包含前 { $count } 个附件；上限为 { $max } 个文件或 { $size }。
mail-focus-not-found = 在 { $folder } 中未找到邮件
mail-focus-go-to = 转到 { $folder }
mail-nav-views-queues = 视图和队列
mail-badge-unread = { $unread ->
   *[other] { $unread } 封未读邮件
  }
mail-badge-total = { $total ->
   *[other] 此文件夹中有 { $total } 封邮件
  }
mail-badge-both = { $unread ->
   *[other] { $unread } 封未读邮件
  }，此文件夹共 { $total ->
   *[other] { $total } 封
  }
mail-badge-drafts = { $total ->
   *[other] 草稿：{ $total } 封邮件
  }
mail-badge-compact-hint = 收件箱以外的文件夹使用较小的徽标。可在“设置”的“文件夹和侧边栏”中更改。
mail-badge-outbox = { $count ->
   *[other] 发件箱：{ $count } 封邮件等待发送
  }
mail-color-menu = 邮件颜色
mail-color-button = 邮件颜色：{ $mode }
mail-color-use-default = 使用我的默认设置（{ $mode }）
mail-color-make-default = 设为我的默认设置
mail-color-locked = 已由管理员锁定
mail-color-confirm-title = 更改默认邮件颜色？
mail-color-confirm-body = 所有邮件都将使用“{ $mode }”，除非你为某封邮件单独更改。
mail-color-confirm = 设为默认
mail-color-default-failed = 无法保存你的默认邮件颜色。

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
mail-attachments-none = 所选邮件没有可供下载的附件。

mail-attachments-zip-all-blocked = 未下载任何内容：附件策略阻止所有 { $skipped } 文件。

mail-attachments-zip-gathering = 正在收集{ $count }条消息的附件...

mail-attachments-zip-skipped = { $skipped ->
       *[other] { $skipped } 个被附件策略阻止的文件被排除。
    }

mail-attachments-zip-unreadable = { $count } 条消息的附件无法阅读并被遗漏。

mail-bulk-download-attachments = 下载附件为ZIP

mail-compose-from-via = 它与 { $header } 标头匹配。

mail-compose-from-why-match-domain = 从 { $identity } 发送，因为邮件已传递到同一域中的地址 { $address }。

mail-compose-from-why-match-pattern = 从 { $identity } 发送，因为消息已发送到 { $address }，这符合该身份的地址模式。

mail-compose-ribbon-autohide = 自动隐藏功能区 (Ctrl+F1)

mail-compose-ribbon-pin = 始终显示功能区 (Ctrl+F1)

mail-context-download-selected-attachments = { $count ->
       *[other] 将 { $count } 条选定邮件的附件下载为 ZIP
    }

mail-context-download-thread-attachments = 将对话的所有 { $count } 附件下载为 ZIP

mail-context-open-new-tab = 在新选项卡中打开

mail-delete-sender-confirm = 将{ $count }邮件移至垃圾箱

mail-delete-sender-detail = 将 { $count } 邮件从 { $sender } 移至垃圾箱？已在垃圾箱中的邮件将被排除。您可以撤消此操作。

mail-delete-sender-empty = 在“已删除邮件”之外没有来自该发件人的邮件。

mail-delete-sender-failed = 无法验证每条匹配的消息及其权限。没有任何东西被移动；再试一次。

mail-delete-sender-help = 在所有文件夹中查找该发件人的邮件，并在确认后将其移至“废纸篓”。

mail-delete-sender-title = 删除该发件人的所有邮件

mail-draft-continue = 继续编辑

mail-draft-edit = 编辑草稿

mail-draft-edit-failed = 无法打开草稿进行编辑

mail-draft-edit-help = 在作曲家中打开此草稿以继续编辑

mail-draft-notice = 这是草稿

mail-file-copied = 复制到剪贴板。

mail-file-copy-failed = 无法复制到剪贴板。

mail-file-copy-fingerprint = 复制指纹

mail-file-copy-name = 复制文件名

mail-file-created = 已创建

mail-file-creator = 创建于

mail-file-details-button = 文件详情

mail-file-details-tooltip = 显示文件详细信息和指纹

mail-file-encryption = 密码保护

mail-file-encryption-none = 没有要求输入密码

mail-file-encryption-password = 需要密码才能打开

mail-file-fingerprint = 指纹

mail-file-name = 文件名

mail-file-pages = 页数

mail-file-pdf-version = PDF版本

mail-file-producer = 制片人

mail-file-verify-help = 将这些组与发件人通过其他方式（例如电话或他们的网站）提供给您的指纹进行比较。如果每个组都匹配，则您将拥有完全相同的字节。

mail-file-verify-title = 验证指纹

mail-follow-up-finished = 后续已完成

mail-group-by = 分组依据

mail-header-attachments = { $count ->
       *[other] { $count } 个附件
    }

mail-header-expand-short = 展开消息标题 (Alt+H)

mail-list-group-date-unknown = 日期未知

mail-list-group-importance = 重要性

mail-list-group-locked = 邮件分组已被管理员锁定。

mail-list-group-no-subject = （无主题）

mail-list-group-none = 没有分组

mail-list-group-priority-high = 高重要性

mail-list-group-priority-low = 重要性低

mail-list-group-priority-normal = 正常重要性

mail-list-group-received = 收到日期

mail-list-group-sender = 来自

mail-list-group-sent = 发送日期

mail-list-group-size = 尺寸

mail-list-group-size-huge = 10MB及以上

mail-list-group-size-large = 1–10 MB

mail-list-group-size-medium = 100 KB–1 MB

mail-list-group-size-small = 25–100 KB

mail-list-group-size-tiny = 小于 25 KB

mail-list-group-subject = 主题

mail-list-order-importance = 重要性，高在先

mail-list-order-importance-asc = 重要性，从低到高

mail-list-order-locked = 消息顺序已被您的管理员锁定。

mail-list-order-sender-desc = 发送者，Z 到 A

mail-list-order-sent-newest = 已发送，最新的优先

mail-list-order-sent-oldest = 已发送，最老的在前

mail-list-order-size = 尺寸，最大的在前

mail-list-order-size-asc = 尺寸，最小的在前

mail-list-order-subject-desc = 主题，Z 到 A

mail-mark-answered = 标记为已回答

mail-mark-as = 标记为

mail-mark-finished = 标记为已完成

mail-mark-forwarded = 标记为已转发

mail-mark-not-forwarded = 标记为未转发

mail-mark-unanswered = 标记为未答复

mail-mentions-me = 提到我

mail-mentions-me-help = 搜索提及您的电子邮件地址之一的邮件

mail-move-search = 搜索文件夹

mail-order-by = 订购依据

mail-reader-folder-go = 存储在 { $folder } · 转到文件夹

mail-reply-draft-discard = 放弃回复

mail-reply-draft-discard-detail = 要永久删除此回复吗？此操作无法撤消。

mail-reply-draft-discard-title = 放弃这个回复？

mail-status-from-match-domain = 从{ $identity }开始

mail-status-from-match-pattern = 从{ $identity }开始

mail-status-from-via = 从{ $identity }回复，因为它匹配{ $header }

mail-toast-follow-up-finished = 后续标记为已完成

mail-toast-marked-answered = { $count ->
       *[other] { $count } 条消息标记为已回复
    }

mail-toast-marked-forwarded = { $count ->
       *[other] { $count } 条消息标记为已转发
    }

mail-toast-marked-not-forwarded = { $count ->
       *[other] { $count } 条消息标记为未转发
    }

mail-toast-marked-unanswered = { $count ->
       *[other] { $count } 条消息标记为未回复
    }

mail-toast-move-failed-reason = 无法移动消息 ({ $reason })。什么都没有动。

mail-toast-update-refused-reason = { $count ->
       *[other] 服务器拒绝移动 { $count } 条消息：{ $reason }
    }

reader-badge-hide = 隐藏此徽章

reader-badge-locked = 由您的管理员设置

reader-badge-menu = 留言徽章

reader-badge-notable = 仅在显着时显示

reader-sanitized-always = 脚本等活动内容保持禁用状态。

reader-sanitized-clean = 消息HTML无需更改

reader-sanitized-clean-detail = 此消息的HTML中没有任何内容被删除或阻止。

reader-sanitized-counts = { $blocked } 远程项目被阻止，{ $trackers } 跟踪项目。

reader-sanitized-detail = 消息HTML在显示之前已被清理。远程内容遵循您的隐私设置。

reader-sanitized-hosts = 远程主机

reader-sanitized-remote = 远程内容被阻止

reader-sanitized-short = 已消毒HTML

reader-sanitized-short-blocked = HTML 被阻止

reader-sanitized-short-clean = HTML 干净

reader-sanitized-trackers = 追踪物品

mail-draft-view = 查看草稿
mail-reply-draft-view = 查看回复
