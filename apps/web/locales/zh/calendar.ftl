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
