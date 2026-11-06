# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-telemetry. Source: ../en/admin-telemetry.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-telemetry-title = 运维遥测
admin-telemetry-help = 将运维追踪和指标导出到兼容 OpenTelemetry 的收集器，例如自托管的 OpenTelemetry Collector。导出默认关闭。保存的更改在重启服务器后生效。
admin-telemetry-back = 返回可观测性
admin-telemetry-saved = 遥测配置已保存。请重启服务器以应用更改。
admin-telemetry-restart = 已保存的配置与正在运行的服务器不同。需要重启。
admin-telemetry-reload = 重新加载配置
admin-telemetry-save = 保存遥测配置
admin-telemetry-collector = 收集器和信号
admin-telemetry-collector-help = 只有管理员可以选择这个服务器范围的目的地。请使用私有收集器或 HTTPS；HTTP 会以未加密方式传输遥测数据。
admin-telemetry-enabled = 启用 OTLP 导出
admin-telemetry-traces = 导出追踪
admin-telemetry-metrics = 导出指标
admin-telemetry-endpoint = OTLP gRPC 端点
admin-telemetry-endpoint-help = 绝对的 HTTP 或 HTTPS 收集器地址，通常使用 4317 端口。不要包含凭据、查询或片段。这是 gRPC 端点，不是 OTLP HTTP 的 4318 端口。
admin-telemetry-serviceName = 服务名称
admin-telemetry-serviceName-help = 在收集器中标识此 Mailwoman 部署的稳定服务标签。请勿使用电子邮件地址或其他个人信息。
admin-telemetry-environment = 环境
admin-telemetry-environment-help = 可选的部署标签，例如 production 或 staging。它会随导出的遥测数据一起发送。
admin-telemetry-pinned = 已禁用的字段由部署环境变量控制。请更改这些变量以覆盖其值。
admin-telemetry-limits = 采样和导出限制
admin-telemetry-limits-help = 限制内存和收集器流量。批大小不能超过队列大小。队列已满时可能会丢弃 span，而不是阻塞邮件请求。
admin-telemetry-sampleRatio = 追踪采样率
admin-telemetry-sampleRatio-help = 保留的新根追踪的比例，范围 0 到 1。子 span 遵循父级的采样决定。
admin-telemetry-exportTimeoutMs = 导出超时（毫秒）
admin-telemetry-exportTimeoutMs-help = 收集器导出请求的最长持续时间，范围 100 到 30000 毫秒。
admin-telemetry-metricsIntervalSecs = 指标间隔（秒）
admin-telemetry-metricsIntervalSecs-help = 两次指标导出之间的时间，范围 10 到 3600 秒。
admin-telemetry-spanQueueSize = Span 队列大小
admin-telemetry-spanQueueSize-help = 等待导出的 span 的最大数量，范围 128 到 32768。
admin-telemetry-spanBatchSize = Span 批大小
admin-telemetry-spanBatchSize-help = 一次导出中的最大 span 数，范围 1 到 4096，且不大于队列大小。
admin-telemetry-spanExportDelayMs = Span 批延迟（毫秒）
admin-telemetry-spanExportDelayMs-help = span 批之间的正常等待时间，范围 100 到 60000 毫秒。
admin-telemetry-prometheus-help = 供兼容 Prometheus 的抓取器使用的独立认证端点。访问始终需要部署的 MW_METRICS_TOKEN 不记名令牌；其值绝不会返回到此页面。
admin-telemetry-prometheusEnabled = 启用 Prometheus 抓取
admin-telemetry-token-missing = 未配置抓取令牌。在运维人员设置 MW_METRICS_TOKEN 之前，指标端点仍无法访问。
