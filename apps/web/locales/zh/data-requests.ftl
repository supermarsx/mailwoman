# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: data-requests. Source: ../en/data-requests.ftl


# -- Machine translation (2026-11): added to match en ---------------------
dr-title = 你的数据
dr-loading = 正在加载
dr-unavailable = 数据请求目前不可用。
dr-off = 此服务器已关闭数据请求。
dr-intro-default = 你可以申请获取你的数据副本、要求更正或删除，或要求我们限制或停止使用你的数据。每项请求都会在其所示的期限内答复。
dr-contact = 数据保护联系人：
dr-kind = 请求类型
dr-kind-access = 访问我的数据
dr-kind-portability = 下载我的数据
dr-kind-rectification = 更正
dr-kind-erasure = 删除
dr-kind-restriction = 限制处理
dr-kind-objection = 反对
dr-kind-other = 其他请求
dr-kind-access-help = 此服务器所持有的关于你的个人数据副本，以及其用途说明。
dr-kind-portability-help = 以机器可读格式（JSON、vCard、iCalendar）打包的 ZIP 文件，可直接迁移到别处。
dr-kind-rectification-help = 要求更正与你有关的数据。
dr-kind-erasure-help = 从此服务器删除你的账户数据。
dr-kind-restriction-help = 要求在争议解决期间保留你的数据但不使用。
dr-kind-objection-help = 反对对你的数据的某项使用。
dr-kind-other-help = 与你的个人数据有关的任何其他请求。
dr-erasure-warning = 删除数据后无法恢复。批准后有 { $days } 天的等待期，期间你可以取消。你的邮件提供商保存的邮件不会被此服务器删除。
dr-erasure-sure = 我了解我的数据将被删除。
dr-rectification-pointer = 你可以在“设置”中自行修改姓名、身份和联系方式。对于无法在那里修改的内容，请使用此请求。
dr-include-mail = 包括存储在此服务器上的我的邮件
dr-details = 详细信息（可选）
dr-reauth = 提交请求之前需要输入你的密码。
dr-submit = 提交请求
dr-submitted = 你的请求已提交。
dr-submitted-code = 你的请求已提交。确认码已发送到你的收件箱；请在下方输入以继续。
dr-error = 出了点问题。请重试。
dr-list = 你的请求
dr-none = 你尚未提交任何请求。
dr-filed = 提交于 { $date }
dr-due = 截止 { $date }
dr-extended = 已延长 { $days } 天
dr-grace = 将在 { $date } 之后执行删除。在此之前你可以取消。
dr-blocked = 此请求已暂停，暂时无法执行。
dr-rejected-reason = 原因：{ $reason }
dr-export-info = 导出文件 { $size } KB，可下载至 { $date }。
dr-export-partial = 部分内容未包含；请查看文件中的清单。
dr-record = 完成记录 { $id }
dr-code = 确认码
dr-code-help = 确认码已发送到你的收件箱。
dr-confirm = 确认
dr-download = 下载
dr-cancel = 取消请求
dr-cancel-erasure = 取消删除
dr-history = 历史记录
dr-status-awaiting_confirmation = 等待你的确认
dr-status-awaiting_verification = 等待身份核验
dr-status-awaiting_approval = 等待批准
dr-status-queued = 排队中
dr-status-processing = 准备中
dr-status-scheduled = 已安排
dr-status-in_progress = 进行中
dr-status-blocked = 已暂停
dr-status-failed = 失败，将重试
dr-status-ready = 已就绪
dr-status-downloaded = 已下载
dr-status-expired = 已过期
dr-status-completed = 已完成
dr-status-rejected = 已拒绝
dr-status-cancelled = 已取消
dr-by-you = 你
dr-by-system = 系统
dr-by-administrator = 管理员
dr-event-submitted = 已提交
dr-event-confirmed = 已确认
dr-event-confirmation-failed = 确认码错误
dr-event-confirmation-expired = 确认已过期
dr-event-awaiting-verification = 等待身份核验
dr-event-awaiting-approval = 等待批准
dr-event-identity-verified = 身份已核验
dr-event-approved = 已批准
dr-event-rejected = 已拒绝
dr-event-blocked-by-hold = 被法律保留阻止
dr-event-hold-set = 已设置法律保留
dr-event-hold-released = 法律保留已解除
dr-event-hold-cleared = 法律保留已清除
dr-event-assigned = 已分配
dr-event-extended = 截止日期已延长
dr-event-cancelled = 已取消
dr-event-completed = 已完成
dr-event-download-link-created = 已创建下载链接
dr-event-downloaded = 已下载
dr-event-export-ready = 导出已就绪
dr-event-export-error = 导出错误
dr-event-export-expired = 导出文件在下载期限后已删除
dr-event-erasure-completed = 删除已完成
dr-event-erasure-error = 删除错误
dr-event-lease-expired = 中断后已恢复处理
dr-event-notice-sent = 已向请求人发送消息
dr-event-notice-failed = 向请求人的消息未能送达
dr-event-due-soon = 即将到期
dr-event-overdue = 已逾期
dr-event-retry = 已重试
dr-event-grace-skipped = 管理员已跳过等待期

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
dr-objection-purposes = 你反对什么

dr-objection-purposes-help = 当管理员批准该请求后，您对您勾选的每个目的的同意将立即撤回。您可以稍后在“同意”下再次打开某个用途。

dr-objection-purposes-list = 反对：{ $purposes }

dr-purpose-analytics = 使用统计

dr-purpose-analyticsTag = 分析标签

dr-purpose-assistant = 助理

dr-purpose-integrations = 集成和API键

dr-purpose-openTracking = 打开并读取跟踪

dr-purpose-push = 推送通知

dr-rectify-banner = 正在更新请求的详细信息{ $ref }

dr-rectify-done = 标记为已完成

dr-rectify-edit = 编辑我的详细信息

dr-rectify-ref = 参考号{ $ref }。如果您写信告知我们有关此更正的信息，请引用该内容。
