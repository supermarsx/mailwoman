# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: calendar. Source: ../en/calendar.ftl

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = 日历
calendar-prev = 上一个
calendar-next = 下一个
calendar-today = 今天
calendar-views = 日历视图
calendar-new-event = 新建日程
calendar-import = 导入
calendar-export = 导出
calendar-import-file = 导入日历文件

# -- View names --------------------------------------------------------------
calendar-view-day = 日
calendar-view-3day = 3 天
calendar-view-work-week = 工作周
calendar-view-week = 周
calendar-view-month = 月
calendar-view-tri-month = 季度
calendar-view-schedule = 日程表
calendar-view-agenda = 议程
calendar-view-year = 年

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = 日历
calendar-toggle = 显示/隐藏 { $name }
calendar-color-for = { $name } 的颜色
calendar-add-calendar = 添加日历
calendar-new-calendar-name = 新日历
calendar-holidays-heading = 节假日
calendar-subscribe-holidays = 订阅节假日
calendar-add-region = 添加地区…
calendar-synced = 已同步的日历

# -- Grid / views ------------------------------------------------------------
calendar-all-day = 全天
calendar-all-day-full = 全天
calendar-more = 另外 { $count } 个
calendar-conflict = 冲突
calendar-no-events-30 = 未来 30 天内没有日程。
calendar-events-count =
    { $count ->
       *[other] { $count } 个日程
    }
# The month grid's accessible name, e.g. "Month of July 2026".
calendar-month-grid = 月视图：{ $month }
# A time-grid's accessible name for the visible day range.
calendar-time-grid = 时间网格
# A single day cell's accessible name: full date + its event count.
calendar-cell = { $date }，{ $events }
# The live-region text announced as focus moves between day cells.
calendar-announce = { $date }，{ $events }
# One event's accessible name: start time + (isolated) title.
calendar-event-at = { $time } { $title }
calendar-event-allday = 全天：{ $title }

# -- Event editor ------------------------------------------------------------
calendar-editor-new = 新建日程
calendar-editor-edit = 编辑日程
calendar-invitation = 邀请
calendar-invited = 你已受邀（{ $status }）。
calendar-accept = 接受
calendar-tentative = 暂定
calendar-decline = 拒绝
calendar-counter = 提议新时间…
calendar-propose-start = 提议新的开始时间
calendar-send-counter = 发送提议
calendar-overlaps = 此日程与其他 { $count } 个日程重叠。
calendar-field-title = 标题
# Stored as the title of an event saved with the title left empty.
calendar-untitled = （无标题）
calendar-field-calendar = 日历
calendar-field-all-day = 全天
calendar-field-start = 开始
calendar-field-duration = 时长（分钟）
calendar-field-location = 地点
calendar-field-shows-as = 显示为
calendar-shows-busy = 忙碌
calendar-shows-free = 空闲
calendar-field-status = 状态
calendar-status-confirmed = 已确认
calendar-status-tentative = 暂定
calendar-status-cancelled = 已取消
calendar-repeats = 重复
calendar-frequency = 频率
calendar-freq-daily = 每天
calendar-freq-weekly = 每周
calendar-freq-monthly = 每月
calendar-freq-yearly = 每年
calendar-every = 每
calendar-interval = 间隔
calendar-ends = 结束
calendar-end-mode = 结束方式
calendar-end-never = 永不
calendar-end-count = N 次后
calendar-end-until = 于指定日期
calendar-occurrences = 次数
calendar-until-date = 结束日期
calendar-reminders = 提醒
calendar-reminder-before = 提前 { $min } 分钟
calendar-remove-reminder = 移除提醒 { $min }
calendar-add-reminder = 添加提醒
calendar-reminder-none = + 提醒
calendar-reminder-5 = 5 分钟
calendar-reminder-15 = 15 分钟
calendar-reminder-30 = 30 分钟
calendar-reminder-60 = 1 小时
calendar-reminder-1440 = 1 天
calendar-attendees = 参与者
calendar-add-attendee = 添加参与者
calendar-attendee-invalid = 这不是电子邮件地址。请输入一个地址，例如 name@example.com。
calendar-save-failed = 日程未保存。服务器拒绝了它。
calendar-remove-attendee = 移除 { $email }
calendar-notes = 备注

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = { $email } 的角色
calendar-attendee-cutype-for = { $email } 的类型
calendar-role-chair = 主持人
calendar-role-required = 必需
calendar-role-optional = 可选
calendar-role-non-participant = 非参与者
calendar-cutype-individual = 个人
calendar-cutype-group = 群组
calendar-cutype-resource = 资源
calendar-cutype-room = 会议室
calendar-partstat-needs-action = 未回复
calendar-partstat-accepted = 已接受
calendar-partstat-declined = 已拒绝
calendar-partstat-tentative = 暂定

# -- Schedule view (distinct from Agenda) ------------------------------------
# Accessible name for one schedule row: start–end time + (isolated) title.
calendar-schedule-row = { $start } 至 { $end }：{ $title }
calendar-schedule-free = 空闲 { $gap }
calendar-gap-hm = { $h } 小时 { $m } 分钟
calendar-gap-h = { $h } 小时
calendar-gap-m = { $m } 分钟

# -- Conflict resolver -------------------------------------------------------
calendar-resolve-conflicts = 解决 { $count } 个冲突
calendar-resolver-title = 解决冲突
calendar-resolver-none = 没有需要解决的冲突。
calendar-resolver-pick = 冲突
calendar-resolver-pair-n = 第 { $n } 个，共 { $total } 个：{ $a } 与 { $b }
calendar-resolver-time = 时间
calendar-resolver-overlap = 重叠时段 { $start } – { $end }
calendar-resolver-reschedule = 重新安排较晚的日程
calendar-resolver-shorten = 缩短较早的日程
calendar-resolver-tentative = 将较晚的日程标记为暂定
calendar-resolver-double-book = 重复预订（标记为空闲）
calendar-resolver-keep = 保留两者
calendar-events-count-people =
    { $count ->
       *[other] { $count } 位参与者
    }

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-whose = 日历
calendar-fb-own = 你的
calendar-fb-unavailable = 无法加载你的忙碌时间。
calendar-fb-cell = { $principal } 在 { $hour }：{ $status }
calendar-fb-busy = 忙碌
calendar-fb-tentative = 暂定
calendar-fb-free = 空闲

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = 快速添加日程
calendar-quick-add-placeholder = 快速添加，例如 Lunch Friday 1pm
calendar-quick-add-btn = 添加
calendar-quick-add-do = 快速添加

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = 类别
calendar-add-category = 添加类别
calendar-category-placeholder = 添加一个类别
calendar-remove-category = 移除类别 { $name }
calendar-filter-heading = 筛选
calendar-filter-category = 按类别筛选
calendar-filter-category-placeholder = 要显示的类别

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = 附件
calendar-attachment-title = 附件名称
calendar-attachment-title-placeholder = 名称（可选）
calendar-attachment-uri = 附件链接
calendar-attachment-add = 添加附件
calendar-remove-attachment = 移除附件 { $name }

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = 通过 URL 订阅
calendar-subscribe-url = 日历 URL
calendar-subscribe-add = 订阅
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = 无法获取该地址的日历，因此未添加。
calendar-subscribe-hol-many-locations = 此节假日文件列出了多个地区。请下载该文件，然后使用“导入”并选择所需的地区。
calendar-subscribe-refresh = 刷新 { $name }
calendar-subscribe-refresh-btn = 刷新
calendar-subscribe-refresh-failed = 无法获取日历订阅源。其日程保持不变。
calendar-import-failed = 无法将该文件作为日历读取。未导入任何内容。
calendar-import-done =
    { $count ->
       *[other] 已导入 { $count } 个日程。
    }
# The location choice shown for a .hol holiday file with more than one location.
calendar-import-locations-title = 选择要导入的地区
calendar-import-locations-intro = 此文件包含 { $count } 个地区的节假日。只会导入你选择的地区。
calendar-import-locations-filter = 筛选地区
calendar-import-locations-list = 文件中的地区
calendar-import-locations-none = 没有符合筛选条件的地区。
calendar-import-locations-picked =
    { $count ->
       *[other] 已选择 { $count } 个地区。
    }
calendar-import-locations-import = 导入所选项
calendar-export-failed = 无法导出日历。
calendar-quick-add-failed = 无法添加该日程。
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = 共享
calendar-share-for = 共享 { $name }
calendar-share-title = 共享 { $name }
calendar-share-intro = 与你共享的人可以查看此日历。授予读写权限后，他们可以添加和编辑日程。
calendar-share-people = 共享对象
calendar-share-empty = 此日历未与任何人共享。
calendar-share-add = 添加人员
calendar-share-new-access = 新人员的权限
calendar-share-access-for = { $principal } 的权限
calendar-share-remove = 移除 { $principal }
calendar-share-read = 只读
calendar-share-readwrite = 读写

# -- Machine translation (2026-11): added to match en ---------------------
calendar-mini = 日期导航
calendar-event-range = { $title },{ $start } 至 { $end }
calendar-busy-free = 显示为空闲
calendar-event-with-status = { $event }({ $status })
calendar-more-label = { $date }还有 { $count } { $count ->
   *[other] 个日程
  }
calendar-fb-caption-people = 冲突当天按小时显示你和每位受邀者的忙碌时间。
calendar-fb-unknown = 无法获取 { $principal } 的空闲状态。
calendar-fb-loading = 正在加载空闲状态…
calendar-import-itip-message = 此文件是通过邮件发送的回复或取消通知。请打开随附的邮件来应用。未导入任何内容。
calendar-import-skipped = { $count ->
   *[other] 有 { $count } 个日程已在日历中,保持原样。
}
calendar-invite-title = 日历邀请
calendar-invite-sender = 来自 { $sender }
calendar-invite-add = 添加到日历
calendar-invite-update = 应用日程更新
calendar-invite-cancel = 应用取消
calendar-invite-reply = 记录参与者回复
calendar-invite-new = 此邀请可以添加到你的日历。
calendar-invite-newer = 组织者发送了更新后的日程。
calendar-invite-current = 你的日历中已有此版本。
calendar-invite-stale = 此邮件比日历中的日程更旧。
calendar-invite-cancelled = 组织者已取消此日程。
calendar-invite-response = 有参与者发来了回复。
calendar-invite-not-organizer = 发件人不是已存储的组织者。不会做任何更改。
calendar-invite-not-attendee = 发件人不是匹配的参与者。不会做任何更改。
calendar-invite-unknown = 此邮件涉及的日程不在你的日历中。
calendar-invite-unsupported = 此提议无法自动应用。
calendar-invite-no-reply = 这会更新你的日历,但不会发送回复。
calendar-invite-applied = 你的日历已更新。
calendar-invite-unchanged = 日程未更改。请查看邀请的当前状态。
calendar-invite-apply-failed = 无法应用此邀请。请重试。
calendar-invite-load-failed = 无法检查此邀请。
calendar-invite-retry = 重试
calendar-field-time-zone = 日程时区
calendar-field-time-zone-help = 使用 IANA 名称,例如 Europe/Lisbon。在其他地方查看时,日程会保留此时区。
calendar-time-invalid = 请输入有效的时区和本地时间。时钟拨快时,部分本地时间并不存在。
calendar-mini-prev = 上个月
calendar-mini-next = 下个月
calendar-card-region-event = 日历日程
calendar-card-kind-request = 会议邀请
calendar-card-kind-update = 更新的邀请
calendar-card-kind-cancel = 已取消的会议
calendar-card-kind-reply = 会议回复
calendar-card-kind-counter = 提议了新时间
calendar-card-kind-publish = { $count ->
   *[other] { $count } 个日历日程
}
calendar-card-when = 时间
calendar-card-where = 地点
calendar-card-organizer = 组织者
calendar-card-attendees = 参与者
calendar-card-all-day = { $when }(全天)
calendar-card-organizer-time = 组织者的时间:{ $when }({ $zone })
calendar-card-repeat-daily = { $interval ->
    [one] 每天
   *[other] 每 { $interval } 天
}
calendar-card-repeat-weekly = { $interval ->
    [one] 每周
   *[other] 每 { $interval } 周
}
calendar-card-repeat-weekly-on = { $interval ->
    [one] 每{ $days }
   *[other] 每 { $interval } 周的{ $days }
}
calendar-card-repeat-monthly = { $interval ->
    [one] 每月
   *[other] 每 { $interval } 个月
}
calendar-card-repeat-monthly-day = { $interval ->
    [one] 每月 { $day } 日
   *[other] 每 { $interval } 个月的 { $day } 日
}
calendar-card-repeat-yearly = { $interval ->
    [one] 每年
   *[other] 每 { $interval } 年
}
calendar-card-repeat-until = { $rule },直到 { $date }
calendar-card-repeat-count = { $rule },{ $count ->
    [one] 一次
   *[other] 共 { $count } 次
}
calendar-card-repeat-custom = 按自定义计划重复
calendar-card-join = 加入 { $provider }
calendar-card-organizer-is = 组织者:{ $who }
calendar-card-people-all = 显示全部 { $count } 位
calendar-card-people-fewer = 显示更少
calendar-card-rsvp-accepted = 已接受
calendar-card-rsvp-tentative = 待定
calendar-card-rsvp-declined = 已拒绝
calendar-card-rsvp-none = 尚未回复
calendar-card-in-calendar = 已在你的日历中
calendar-card-cancelled = 已取消
calendar-card-open = 在日历中打开
calendar-card-agenda = 你在 { $day } 的日历
calendar-card-agenda-title = 你的一天
calendar-card-agenda-this-event = 此日程
calendar-card-agenda-unavailable = 无法加载你这一天的日历。
calendar-card-conflicts = { $count ->
    [one] 与 { $titles } 冲突
   *[other] 与 { $count } 个日程冲突:{ $titles }
}
calendar-card-no-conflicts = 你的日历中没有冲突
calendar-card-invited-as = 你受邀的地址是 { $address }。
calendar-card-not-invited = 此邀请未列出你的任何地址。你可以将其添加到日历,但无法回复。
calendar-card-not-invited-short = 此邀请未列出你的任何地址。
calendar-card-out-of-date = 此邀请已过期。你的日历中已有更新的版本。
calendar-card-replied = { $who }{ $answer ->
    [accepted] 已接受。
    [tentative] 已暂时接受。
    [declined] 已拒绝。
   *[other] 尚未回复。
}
calendar-card-counter = { $who } 提议了新的时间。提议不会自动应用。
calendar-card-todos-skipped = { $count ->
    [one] 该文件还包含 1 个任务,未添加。
   *[other] 该文件还包含 { $count } 个任务,未添加。
}
calendar-card-respond = 回复邀请
calendar-card-accept = 接受
calendar-card-tentative = 待定
calendar-card-decline = 拒绝
calendar-card-propose = 提议新时间
calendar-card-propose-start = 新的开始时间
calendar-card-propose-send = 发送提议
calendar-card-propose-invalid = 请输入有效的新开始时间。
calendar-card-send-response = 向 { $organizer } 发送回复
calendar-card-comment = 备注(可选)
calendar-card-calendar = 日历
calendar-card-events = 此文件中的日程
calendar-card-add-one = 添加
calendar-card-add-all = 全部添加({ $count })
calendar-card-added = { $count ->
    [one] 已添加到你的日历。
   *[other] 已将 { $count } 个日程添加到你的日历。
}
calendar-card-remove = 从日历中移除
calendar-card-removed = 已从你的日历中移除。
calendar-card-record-reply = 更新参与者状态
calendar-card-answered-sent = { $answer }。你的回复已发送给 { $organizer }。
calendar-card-answered-not-sent = { $answer }。你的日历已更新,但回复未能发送。
calendar-card-answered-silently = { $answer }。未发送回复。
calendar-card-proposal-sent = 你的提议已发送给 { $organizer }。
calendar-card-not-sent = 邮件未能发送。
calendar-card-more = { $count ->
    [one] 另有 1 个
   *[other] 另有 { $count } 个
}

# -- Machine translation (2026-11): added to match en ---------------------
calendar-card-repeat-except = { $count ->
    [one] { $rule }(跳过 1 个日期)
   *[other] { $rule }(跳过 { $count } 个日期)
}
calendar-card-attendee-summary = { $count ->
    [one] 1 位参与者 · 已接受 { $accepted },待定 { $tentative },已拒绝 { $declined }
   *[other] { $count } 位参与者 · 已接受 { $accepted },待定 { $tentative },已拒绝 { $declined }
}
