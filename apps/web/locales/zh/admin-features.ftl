# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-features. Source: ../en/admin-features.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-features-title = 功能配置文件
admin-features-intro = 配置文件是一组具名的功能开关。可将其分配给域名、群组或用户。按个人分配支持原生邮箱以及经过验证的代理或 SSO 身份；它们控制对 Mailwoman 功能的访问。
admin-features-new = 新建配置文件
admin-features-search = 搜索配置文件
admin-features-empty = 还没有功能配置文件。
admin-features-no-match = 没有配置文件符合搜索条件。
admin-features-load-error = 无法加载功能目录。
admin-features-not-loaded = 功能目录尚未加载。请重新加载页面后再试。
admin-features-stale = 另一位管理员更改了功能目录。已重新加载；请检查各项值后再次保存。
admin-features-not-found = 没有 id 为 { $id } 的配置文件。
admin-features-col-name = 名称
admin-features-col-values = 开关
admin-features-col-assigned = 分配给
admin-features-values-count = { $count ->
   *[other] 已设置 { $count } 个开关
}
admin-features-assigned-count = { $count ->
    [0] 未分配
   *[other] { $count } 处
}
admin-features-editor = 配置文件设置
admin-features-name = 配置文件名称
admin-features-name-help = 仅向管理员显示，最多 128 个字符。
admin-features-name-invalid = 请输入 1 至 128 个字符的名称。
admin-features-switches = 功能开关
admin-features-switches-help = “未设置”会把该功能交给下一个配置文件、域名或服务器默认值决定。锁定的值不能由用户或更低级别更改。
admin-features-col-feature = 功能
admin-features-col-value = 值
admin-features-col-lock = 锁定
admin-features-value-for = { $feature } 的值
admin-features-unset = 未设置
admin-features-on = 允许
admin-features-off = 不允许
admin-features-lock = 已锁定
admin-features-save = 保存配置文件
admin-features-create = 创建配置文件
admin-features-delete = 删除配置文件
admin-features-delete-title = 删除配置文件 { $name }？
admin-features-delete-body = { $count ->
    [0] 此配置文件未分配到任何位置。其开关将立即停止生效。
   *[other] 此配置文件将从其分配到的 { $count } 处移除，其开关将立即停止生效。
}
admin-features-saved = 配置文件 { $name } 已保存。
admin-features-deleted = 配置文件 { $name } 已删除。
admin-features-assigned = 配置文件 { $name } 已分配。
admin-features-unassigned = 已移除 { $name } 的分配。
admin-features-assignments = 分配给
admin-features-assignments-help = 配置文件按优先级顺序生效，优先级高的在前。优先级相同时，用户分配先于群组，群组先于域名。
admin-features-assignments-empty = 此配置文件尚未分配。
admin-features-assign-form = 分配此配置文件
admin-features-col-scope = 适用于
admin-features-col-target = 名称
admin-features-col-priority = 优先级
admin-features-col-actions = 操作
admin-features-scope-domain = 域名
admin-features-scope-group = 群组
admin-features-scope-user = 用户
admin-features-assign = 分配
admin-features-unassign = 移除
admin-features-no-domain = 尚未配置域名。
admin-features-no-group = 不存在用户群组。
admin-features-no-user = 还没有邮箱账户登录过。
admin-features-tools = 配置文件工具
admin-features-preset = 从预设开始
admin-features-preset-choose = 选择预设
admin-features-preset-apply = 应用预设
admin-features-preset-applied = 已应用预设 { $name }。请检查各项值后保存。
admin-features-preset-full = 完整
admin-features-preset-standard = 标准
admin-features-preset-mail-only = 仅邮件
admin-features-preset-minimal = 极简 / 展台
admin-features-import = 导入 JSON
admin-features-export = 导出 JSON
admin-features-exported = 配置文件已导出为 JSON 文件。
admin-features-imported = { $count ->
   *[other] 已导入 { $count } 个值。请检查后保存。
}
admin-features-import-json = 该文件不是有效的 JSON。
admin-features-import-format = 该文件不是 Mailwoman 功能配置文件。
admin-features-import-version = 此功能配置文件来自更新的版本，无法导入。
admin-features-clone = 克隆
admin-features-clone-name = { $name } 的副本
admin-features-filter = 查找功能
admin-features-filter-placeholder = 名称、设置或键
admin-features-filter-none = 没有功能符合搜索条件。
admin-features-diff-summary = { $differs ->
   *[other] { $differs } 个值与服务器默认值不同
} · { $same } 个与之相同
admin-features-differs = 与默认值不同
admin-features-default-is = 默认值：{ $value }
admin-features-fix-values = 保存前请更正标记为无效的值。
admin-features-allow-all = 全部允许
admin-features-deny-all = 全部不允许
admin-features-clear-all = 清除
admin-features-group-allow = 允许 { $group } 中的所有功能
admin-features-group-deny = 不允许 { $group } 中的所有功能
admin-features-group-clear = 清除 { $group } 中的所有值
admin-features-enforcement-ui = 仅在应用中隐藏
admin-features-enforcement-partial = 部分限制尚未计数
admin-features-other = 配置文件可包含的其他设置
admin-features-other-help = 管理员可按配置文件设置的所有其他设置。字段留空则保持服务器的值。
admin-features-cat-compose = 邮件和撰写
admin-features-cat-reading = 阅读和隐私
admin-features-cat-views = 视图和搜索
admin-features-cat-organise = 整理
admin-features-cat-pim = 日历、联系人、任务、笔记和密钥
admin-features-cat-data = 数据
admin-features-cat-accounts = 账户和访问
admin-features-cat-integrations = 集成
admin-features-cat-developer = 开发者访问
admin-features-cat-personal = 个性化
admin-features-feature-compose-sending = 发送限制
admin-features-feature-compose-sending-help = 每封邮件的收件人数超过限制时，服务器会拒绝。每小时和每天的邮件数会为配置文件存储，但尚未计数。
admin-features-feature-storage = 存储配额
admin-features-feature-storage-help = 存储限制会为配置文件存储；邮件服务器自身的配额仍然适用。
admin-features-feature-access-protocols = 邮件访问协议
admin-features-feature-access-protocols-help = 这些用户可使用哪些邮件服务器协议登录，以逗号分隔：imap, pop3, ews。
admin-features-feature-accounts-linked = 已关联的邮箱
admin-features-feature-calendar = 日历
admin-features-feature-contacts = 联系人
admin-features-feature-tasks = 任务
admin-features-feature-notes = 笔记
admin-features-preview = 生效的值
admin-features-preview-help = 在计入每个配置文件、群组、域名的值和锁定之后，每项功能对单个用户或某个域名的用户最终得到的结果。
admin-features-preview-kind = 预览对象
admin-features-preview-target = 对象
admin-features-preview-show = 显示
admin-features-preview-loading = 正在加载…
admin-features-preview-state = 生效
admin-features-preview-from = 决定方
admin-features-preview-profiles = 适用的配置文件，先匹配者优先：{ $names }
admin-features-preview-no-profiles = 没有适用的配置文件；由域名和服务器的值决定。
admin-features-source-default = 内置默认值
admin-features-source-env-fallback = 环境默认值
admin-features-source-env-pin = 环境（固定）
admin-features-source-global = 服务器值
admin-features-source-domain = 域名值
admin-features-source-profile = 配置文件
admin-features-source-profile-named = 配置文件 { $name }
admin-features-source-group = 群组值
admin-features-source-admin-user = 为此用户设置的值
admin-features-source-user = 用户自己的选择
admin-features-source-locked = { $source }（已锁定）
admin-retention-title = 审计日志保留
admin-retention-intro = 每个类别的审计条目保留多久。没有自己的值的类别沿用默认值；没有默认值时保留所有条目。法律保留会停止其类别的清理，对默认值设置法律保留会停止所有类别的清理。更改和清理都会记录在审计日志中。
admin-retention-loading = 正在加载保留策略…
admin-retention-unavailable = 此服务器不提供审计保留。
admin-retention-col-category = 类别
admin-retention-col-kept = 保留
admin-retention-col-change = 更改
admin-retention-cat-default = 默认
admin-retention-cat-admin = 管理员操作
admin-retention-cat-security = 安全事件
admin-retention-cat-alert = 用户警报
admin-retention-forever = 所有条目
admin-retention-days = { $days ->
   *[other] { $days } 天
}
admin-retention-follows-default = 默认：{ $kept }
admin-retention-hold-on = 法律保留
admin-retention-read-only = 更改保留设置需要 settings.write 权限。
admin-retention-edit-for = { $category } 的保留设置
admin-retention-keep = 保留
admin-retention-mode-inherit = 沿用默认值
admin-retention-mode-forever = 所有条目
admin-retention-mode-days = 指定天数
admin-retention-days-label = 天数
admin-retention-hold = 法律保留
admin-retention-save = 保存
admin-retention-bounds = 请输入 { $min } 至 { $max } 之间的整数。
admin-retention-saved = 已保存 { $category } 的保留设置。
admin-retention-task-none = 还没有清理任务。首次保存天数时会创建，每天 03:15 UTC 运行。
admin-retention-task-disabled = 计划的清理任务已停用；只有使用“立即清理”才会清理条目。
admin-retention-task-next = 下次计划清理：{ $when }。
admin-retention-task-daily = 每天 03:15 UTC 清理。
admin-retention-last = 上次删除了条目的清理：{ $when }。
admin-retention-never = 尚未清理任何条目。
admin-retention-prune = 立即清理
admin-retention-prune-title = 立即清理审计日志？
admin-retention-prune-body = 早于其类别保留期限的条目将被删除。保留所有条目或处于法律保留的类别不会更改。此操作无法撤销。
admin-retention-pruned = { $count ->
    [0] 没有足够旧的条目可删除。
   *[other] 已删除 { $count } 个条目。
}
admin-auth-attempted = 尝试的身份：{ $name }
admin-features-feature-compose-open-tracking = 邮件打开跟踪
admin-features-feature-compose-open-tracking-help = 允许使用服务器的跟踪图片记录符合条件的格式化邮件的打开情况。需要已配置公开 URL。
admin-features-feature-reading-original-html = 原始 HTML 视图
admin-features-feature-reading-original-html-help = 允许用户在隔离的阅读框架内，为个别邮件选择使用原始 HTML。默认仍为净化后的 HTML。
admin-features-inherit = 继承
admin-features-override = 设置值
admin-features-setting-mode = { $setting } 的取值方式
admin-features-inherit-help = 继承会移除此配置文件的值和锁定。由下一个适用的配置文件、域名或服务器的值决定；生效预览会显示结果及其来源。
admin-features-domain-alias = { $alias }（{ $domain } 的别名）
admin-features-domain-aliases = { $domain }（别名：{ $aliases }）
