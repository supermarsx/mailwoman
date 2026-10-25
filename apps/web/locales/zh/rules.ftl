# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: rules. Source: ../en/rules.ftl

rules-title = 规则和过滤器
rules-intro = 管理邮件过滤脚本和可视化规则。可用功能取决于此账户和管理员设置。
rules-empty = 还没有规则。
rules-unavailable = 此邮件服务器不支持邮件规则。
rules-load-failed = 无法加载规则。
rules-new = 新建规则
rules-list-label = 邮件规则
rules-enabled = 已启用
rules-delete-named = 删除规则 { $name }

# Builder
rules-builder-title = 规则构建器
rules-builder-hint = 选择要编辑的规则，或新建一条规则。
rules-name-label = 规则名称
rules-match-legend = 运行时机
rules-match-all = 满足所有条件
rules-match-any = 满足任一条件
rules-conditions-legend = 条件
rules-actions-legend = 操作
rules-add-condition = 添加条件
rules-add-action = 添加操作
rules-remove-condition = 删除条件
rules-remove-action = 删除操作
rules-cond-type-label = 字段
rules-cond-op-label = 运算符
rules-cond-value-label = 匹配值
rules-act-type-label = 操作
rules-act-value-label = 操作值
rules-save = 保存规则
rules-editing = 正在编辑 { $name }
rules-save-refused = 规则未保存：{ $reason }

# Condition fields and operators
rules-cond-from = 发件人
rules-cond-to = 收件人
rules-cond-subject = 主题
rules-cond-thread-unsupported = 会话 { $op } { $value }：不支持会话条件，因此在删除此条件之前无法保存规则。
rules-op-contains = 包含
rules-op-is = 是

# Actions
rules-act-move = 移动到文件夹
rules-act-tag = 添加标签
rules-act-archive = 归档
rules-act-suppress = 不显示通知
rules-act-stop = 停止处理

# Where it runs
rules-runs-server = 服务器（Sieve）
rules-runs-server-detail = 上传到邮件服务器并在服务器上应用。
rules-runs-engine = 引擎
rules-runs-engine-detail = 在同步新收件时应用；回复和移动将在下次同步时执行。

# Panes
rules-panes-label = 规则工具
rules-tab-builder = 构建器
rules-tab-raw = 原始 Sieve
rules-tab-dryrun = 试运行

# Raw editor
rules-raw-label = 原始 Sieve 源代码
rules-raw-reset = 重置为生成的代码
rules-lint-clean = 未发现问题。

# Dry run
rules-dryrun-title = 试运行
rules-dryrun-help = 输入一封示例邮件，预览哪些规则会被触发。
rules-dryrun-results = 试运行结果
rules-dryrun-none = 没有可评估的已启用规则。
rules-dryrun-nomatch = 不匹配
rules-dryrun-stopped = 未执行到（之前的规则已停止处理）

rules-scripts-label = Sieve 脚本
rules-script-active = 已启用
rules-script-new = 新建脚本
rules-script-export = 导出脚本
rules-script-import = 导入脚本
rules-script-too-large = 脚本超过大小限制。
rules-script-read-only = 管理员已禁用脚本直接编辑。
rules-script-validate = 验证脚本
rules-script-deactivate = 停用
rules-script-activate = 启用
rules-script-saved = 脚本已保存。

rules-vacation-title = 休假回复
rules-vacation-enabled = 启用自动回复
rules-vacation-from = 开始时间（UTC）
rules-vacation-to = 结束时间（UTC）
rules-vacation-body = 回复正文
rules-vacation-saved = 休假回复已保存。
rules-script-template = 脚本模板
rules-forward-title = 转发
rules-forward-address = 转发至
rules-forward-copy = 在此账户中保留副本
rules-forward-refused = 管理员不允许转发到此域名。
