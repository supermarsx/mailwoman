# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-stats. Source: ../en/admin-stats.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-stats-key-figures = 关键数据
admin-stats-charts = 图表
admin-stats-tables = 表格
admin-stats-group-traffic = 此期间的流量
admin-stats-group-storage = 当前的存储与投递
admin-stats-refresh = 刷新统计
admin-stats-loading = 正在加载统计…
admin-stats-requests-total = 请求
admin-stats-errors-total = 服务器错误（5xx）
admin-stats-p95-peak = 最慢的第 95 百分位
admin-stats-requests = 请求
admin-stats-errors = 服务器错误（5xx）
admin-stats-not-found = 未找到（404）
admin-stats-latency = 响应时间
admin-stats-series-p50 = 中位数
admin-stats-series-p95 = 第 95 百分位
admin-stats-unit-count = 每个{ $bucket }的数量
admin-stats-unit-duration = 每个{ $bucket }的时间，使用所示单位
admin-stats-unit-accounts = 每个域名被阻止的账户
admin-stats-chart-about = 从 { $from } 到 { $to } 的折线图。
admin-stats-bars-about = { $count ->
   *[other] 包含 { $count } 个域名的条形图。
}
admin-stats-view-table = 以表格查看
admin-stats-legend = 图例
admin-stats-empty-chart = 此期间尚未测量到任何数据。请选择更长的时间段，或在服务器处理了一些请求后再来查看。
admin-stats-empty-blocked = 没有被阻止的账户。
admin-stats-delta-vs = 与上一时间段相比
admin-stats-delta-same = 与上一时间段相同
admin-stats-delta-sr-up = 比上一时间段增加 { $change }。
admin-stats-delta-sr-down = 比上一时间段减少 { $change }。
admin-stats-delta-sr-same = 与上一时间段相同。
admin-ct-group = 客户端加载时间
admin-ct-metric-timeToUsable = 可用时间
admin-ct-metric-ttfb = 首字节
admin-ct-metric-dcl = DOM 就绪
admin-ct-metric-load = 页面加载
admin-ct-metric-fcp = 首次内容绘制
admin-ct-metric-lcp = 最大内容绘制
admin-ct-metric-folderOpen = 打开文件夹
admin-ct-metric-folderLoadCold = 文件夹加载，来自网络
admin-ct-metric-folderLoadCached = 文件夹加载，来自缓存
admin-ct-metric-messageLoadCold = 邮件加载，来自网络
admin-ct-metric-messageLoadCached = 邮件加载，来自缓存或预取
admin-ct-device-desktop = 桌面设备
admin-ct-device-mobile = 移动设备
admin-ct-device-unknown = 未知
admin-ct-tile = { $metric }，{ $stat }
admin-ct-samples =
    { $count ->
       *[other] { $count } 个样本
}
admin-ct-no-samples = 此期间没有样本。
admin-ct-few-samples = 只有 { $count } 个样本；至少需要 { $min } 个。
admin-ct-chart-usable = 可用时间
admin-ct-group-folder = 文件夹加载时间
admin-ct-group-message = 邮件加载时间
admin-ct-chart-folder-cold = 从网络加载文件夹
admin-ct-chart-folder-cached = 从缓存加载文件夹
admin-ct-chart-message-cold = 从网络加载邮件
admin-ct-chart-message-cached = 从缓存或预取加载邮件
admin-ct-folder-off = 文件夹加载时间默认关闭。只测量已开启它的用户。
admin-ct-message-off = 邮件加载时间默认关闭。只测量已开启它的用户。
admin-ct-chart-note = 仅在样本数至少为 { $min } 的区间内显示百分位数。
admin-ct-empty-chart = 没有任何区间达到显示中位数所需的 { $min } 个样本。请选择更长的时间段。
admin-ct-col-samples = 样本
admin-ct-col-metric = 指标
admin-ct-col-device = 设备
admin-ct-table-title = 按设备划分的客户端加载时间
admin-ct-table-caption = 每项指标和设备类别的样本数、中位数和第 95 百分位。
admin-ct-empty-table = 此期间没有报告任何浏览器加载时间。
admin-ct-failed = 无法加载客户端加载时间。
admin-ct-disabled = 客户端加载时间的收集已关闭。下方数据来自其关闭之前。
admin-ct-note = 仅来自已登录的浏览器，抽样 { $rate }% 的会话，匿名且经过汇总。可用时间从导航开始，直到邮件列表显示第一批行。数据为估计值，与真实值相差约 { $error }% 以内。
