# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-privacy-speed. Source: ../en/admin-privacy-speed.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-speed-title = 速度
privacy-speed-unavailable = 此服务器无法提供速度数据。
privacy-speed-empty = 此范围内尚未提供任何图片。
privacy-speed-off = 速度跟踪已关闭（设置：图片代理速度跟踪）。不会记录任何新数据。
privacy-speed-verdict-healthy = 良好
privacy-speed-verdict-slow = 较慢
privacy-speed-verdict-degraded = 性能下降
privacy-speed-verdict-noData = 数据不足
privacy-speed-reason-ok = 抓取的图片和缓存命中均在下列限值内，且失败的请求很少。
privacy-speed-reason-failureRate = { $failurePct } % 的请求以繁忙、超时或上游失败结束。
privacy-speed-reason-missP95 = 需要抓取的图片较慢：其第 95 百分位高于下列限值。
privacy-speed-reason-hitP95 = 缓存图片响应较慢：其第 95 百分位高于下列限值。
privacy-speed-reason-fewRequests = 此范围内的请求少于 { $min } 个，样本太少，无法判断。
privacy-speed-thresholds = 较慢的条件：抓取图片第 95 百分位达到 { $slowMiss }、缓存图片第 95 百分位达到 { $slowHit }，或 { $slowFail } % 的请求繁忙、超时或失败。性能下降的条件：抓取图片第 95 百分位达到 { $degradedMiss }，或 { $degradedFail } % 的请求失败。限值固定，不可设置。
privacy-speed-median = 中位数
privacy-speed-p95 = 第 95 百分位
privacy-speed-p99 = 第 99 百分位
privacy-speed-max = 最慢
privacy-speed-requests = 已测量的请求
privacy-speed-busy = 繁忙应答
privacy-speed-timeouts = 超时
privacy-speed-upstream-errors = 上游错误
privacy-speed-hit = 缓存命中
privacy-speed-miss = 抓取的图片（缓存未命中）
privacy-speed-split-count = { $count } 个请求
privacy-speed-split-times = 中位数 { $p50 }，第 95 百分位 { $p95 }，最慢 { $max }。
privacy-speed-chart = 提供图片所用时间
privacy-speed-chart-miss = 提供抓取图片所用时间
privacy-speed-chart-unit = 从收到请求到响应就绪
privacy-speed-phases = 时间花在哪里
privacy-speed-phases-help = 针对未命中缓存的请求：每个步骤花费的时间及其占总时间的比例。
privacy-speed-phase = 步骤
privacy-speed-share = 时间占比
privacy-speed-phase-queue = 等待容量
privacy-speed-phase-cache = 缓存查找
privacy-speed-phase-upstream = 上游抓取
privacy-speed-phase-decode = 解码和重新编码
privacy-speed-phase-other = 其他所有
privacy-speed-note = 仅在此服务器进程上测量，保存在内存中，重启后清空。百分位的误差在 { $error } % 以内。不存储任何 URL、主机或账户。

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-work-coalesced = 联合获取和优先级请求

privacy-work-coalesced-series = 获取连接到飞行中的一个

privacy-work-handle = 处理缓存命中率

privacy-work-handle-series = 处理缓存命中

privacy-work-per-interval = 每个采样间隔

privacy-work-percent = 句柄查找的份额

privacy-work-priority-series = 优先请求

privacy-work-resized = 图像缩小

privacy-work-resized-series = 调整大小

privacy-work-revalidated-series = 重新验证

privacy-work-saved = 通过缩减规模节省的字节数

privacy-work-saved-series = 已保存字节数

privacy-work-stale = 提供过时的图像并重新验证

privacy-work-stale-series = 服务陈旧

privacy-work-title = 工作被避免和重塑
