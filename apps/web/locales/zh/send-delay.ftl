# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: send-delay. Source: ../en/send-delay.ftl


# -- Machine translation (2026-11): added to match en ---------------------
send-delay-title = 撤销发送
send-delay-help = 按下“发送”后，邮件会在发件箱中等待这么长时间。在此之前，你可以收回并继续编辑。邮件由服务器保管，因此关闭浏览器不会提前发送，也不会丢失。
send-delay-label = 已发送邮件保留时长
send-delay-off = 关闭（立即发送）
send-delay-seconds = { $n } 秒
send-delay-minutes = { $n ->
   *[other] { $n } 分钟
}
send-delay-custom = 自定义…
send-delay-custom-label = 自定义延迟（秒）
send-delay-custom-help = 整数秒，0 到 { $max }。
send-delay-capped = 管理员将此项限制为 { $max }。
send-delay-disabled = 管理员已关闭收回已发送邮件的功能。邮件将立即发送。
send-delay-invalid = 请输入 0 到 { $max } 之间的整数秒数。
send-later-button = 稍后发送
send-later-menu = 稍后发送选项
send-later-preset-hour = 1 小时后
send-later-preset-today = 今天晚些时候，17:00
send-later-preset-tomorrow = 明天上午，09:00
send-later-preset-monday = 周一上午，09:00
send-later-custom = 选择日期和时间…
send-later-custom-label = 发送时间
send-later-zone = 时区：{ $zone }
send-later-schedule = 定时发送
send-later-cancel = 取消
send-later-err-invalid = 请输入有效的日期和时间。
send-later-err-past = 请选择未来的时间。
send-later-err-horizon = 时间太远了。邮件最多可提前 { $days } 天定时发送。
send-later-err-off = 管理员已关闭定时发送。
send-queued = 邮件已排队，发送倒计时：
send-queued-undo = 撤销
send-queued-now = 立即发送
send-queued-recalled = 邮件已收回，已回到草稿中。
send-queued-too-late = 来不及收回：邮件已在发送中。
send-queued-sent-now = 正在发送。
send-queued-scheduled = 已安排在 { $when }（{ $zone }）发送。
outbox-scheduled-badge = 已定时
outbox-sends-in = { $time }后发送
outbox-sends-at = 发送时间：{ $when }
outbox-recall-edit = 收回并编辑
outbox-recall-too-late = 已在发送中，无法再收回。
outbox-failed-drafts = 无法发送，已保存在你的草稿中。
outbox-queued-count = { $count ->
   *[other] { $count } 封邮件等待发送
}

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
outbox-chip-next = 下一次发送：{ $when }

outbox-chip-title = 等待发送的消息

outbox-chip-view-all = 打开发件箱

outbox-delete = 删除

outbox-delete-confirm = 删除消息？

outbox-delete-failed = 该消息无法删除。 { $error }

outbox-deleted = 排队的消息已被删除。

outbox-edit-time = 编辑发送时间

outbox-edit-time-conflict = 发送时间已更改或消息已发送。发件箱已重新加载。

outbox-edit-time-current = 目前{ $when }

outbox-edit-time-refused = 发送时间无法更改。 { $error }

outbox-edit-time-save = 节省发送时间

outbox-failed-toast = 无法发送消息。 { $error }

outbox-failed-uncertain-toast = 消息可能已发送，也可能未发送。在再次发送之前检查已发送。

outbox-open = 打开

outbox-reader-actions = 排队的消息操作

outbox-reader-cc = 抄送：{ $recipients }

outbox-reader-label = 排队消息

outbox-reader-loading = 正在加载消息...

outbox-reader-to = 致：{ $recipients }

outbox-reader-unavailable = 无法加载该消息。

outbox-recall-to-drafts = 召回草稿

outbox-send-again = 再次发送

outbox-send-again-failed = 无法再次将消息排队。 { $error }

outbox-send-again-queued = 该消息已排队等待再次发送。

outbox-sent-copy-folder = 将保存在{ $folder }

outbox-sent-copy-missing = 将保存在已发送中（所选文件夹不可用）

outbox-sent-copy-skip = 不会保存任何副本

outbox-status-attempts = { $count ->
       *[other] 到目前为止 { $count } 次尝试
    }

outbox-status-canceled = 取消

outbox-status-failed = 未发送

outbox-status-held = 保持直到您释放为止

outbox-status-last-error = 最后一个错误：{ $error }

outbox-status-next-attempt = 下次尝试{ $when }

outbox-status-queued = 排队

outbox-status-retrying = 重试

outbox-status-scheduled = 预定

outbox-status-sending = 发送

outbox-status-sends-at-in = 发送{ $when }（在{ $time }中）

outbox-status-sent = 已发送

outbox-status-uncertain = 交货未知
