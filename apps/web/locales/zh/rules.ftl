# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: rules. Source: ../en/rules.ftl

rules-title = 规则和过滤器
rules-intro = 自动对收到的邮件进行分类、添加标签或执行操作。仅使用标准条件的规则以 Sieve 形式在邮件服务器上运行；其余规则在投递时由引擎运行。
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
rules-runs-engine-detail = 在投递时由引擎应用。

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
