# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-webhooks. Source: ../en/admin-webhooks.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-webhooks-title = Webhook
admin-webhooks-intro = 出站 webhook 会在某个账户发生事件时，向一个 URL 发送带签名的通知。入站 webhook 允许另一个系统在某个账户中执行邮件规则操作。
admin-webhooks-empty = 暂无 webhook。
admin-webhooks-unavailable = 此服务器不提供 webhook 管理。
admin-webhooks-load-error = 无法加载 webhook。
admin-webhooks-direction-outbound = 出站
admin-webhooks-direction-inbound = 入站
admin-webhooks-col-direction = 方向
admin-webhooks-col-label = 名称
admin-webhooks-col-account = 账户
admin-webhooks-col-target = 发送到或允许
admin-webhooks-col-created = 创建时间
admin-webhooks-no-label = （无名称）
admin-webhooks-created-by = 创建者：{ $who }
admin-webhooks-last-used = 上次使用：{ $when }
admin-webhooks-never-used = 尚未使用
admin-webhooks-delete = 删除
admin-webhooks-delete-named = 删除 webhook { $name }
admin-webhooks-delete-title = 删除 webhook { $name }？
admin-webhooks-delete-body = 向它的投递将停止，或使用其密钥签名的调用会被拒绝。此操作无法撤销；新的 webhook 会获得新的密钥。
admin-webhooks-deleted = Webhook 已删除。
admin-webhooks-add = 添加 webhook
admin-webhooks-add-direction = 方向
admin-webhooks-add-outbound-help = Mailwoman 调用你的 URL。
admin-webhooks-add-inbound-help = 你的系统调用 Mailwoman。
admin-webhooks-add-account = 账户
admin-webhooks-add-account-help = 其事件会被发送的账户，或入站调用唯一可以操作的账户。
admin-webhooks-add-label = 名称
admin-webhooks-add-label-help = 此 webhook 的用途，方便他人识别。可选。
admin-webhooks-add-url = URL
admin-webhooks-add-url-help = 必须能访问公共互联网地址。私有、回环和链路本地地址会被拒绝。
admin-webhooks-add-events = 事件
admin-webhooks-add-actions = 允许的操作
admin-webhooks-add-submit = 创建 webhook
admin-webhooks-add-saving = 正在创建…
admin-webhooks-add-check = 检查
admin-webhooks-add-check-ok = URL 通过了检查。未保存任何内容。
admin-webhooks-event-message-received = 新邮件
admin-webhooks-event-message-flags = 邮件已读、已加标记或已加标签
admin-webhooks-event-message-moved = 邮件已移动
admin-webhooks-event-message-deleted = 邮件已删除
admin-webhooks-event-submission-sent = 邮件已发送
admin-webhooks-event-submission-failed = 邮件无法发送
admin-webhooks-event-pim-changed = 日历、联系人、任务或笔记已更改
admin-webhooks-action-run-rules = 对邮件描述运行该账户的邮件规则
admin-webhooks-action-sieve = 对邮件应用一项规则操作
admin-webhooks-secret-title = { $name } 的密钥
admin-webhooks-secret-once = 请立即复制此密钥。它不会再次显示。
admin-webhooks-secret-outbound = 每次投递都带有 X-Mailwoman-Signature: sha256=，后面是使用此密钥对请求正文计算的 HMAC-SHA256。
admin-webhooks-secret-inbound = 发送 X-Mailwoman-Webhook: { $id }、X-Mailwoman-Timestamp（Unix 秒）、X-Mailwoman-Nonce（16 到 128 个字母、数字、- 或 _，不可重复使用）以及 X-Mailwoman-Signature: sha256=，后面是使用此密钥对 timestamp.nonce.body 计算的 HMAC-SHA256。时间偏差超过五分钟或重复使用 nonce 的调用会被拒绝。
admin-webhooks-secret-done = 我已复制
admin-webhooks-error-stale = 自你打开此页面以来，有人更改了 webhook。列表已重新加载；请重试。
admin-webhooks-error-account = 请输入账户。
admin-webhooks-error-url = 请输入以 https:// 或 http:// 开头的 URL。
admin-webhooks-error-events = 请至少选择一个事件。
admin-webhooks-error-actions = 请至少选择一项操作。
