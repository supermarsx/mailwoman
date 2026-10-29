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
mail-toast-pinned = 已置顶
mail-toast-unpinned = 已取消置顶
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
