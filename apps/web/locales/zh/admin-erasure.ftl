# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-erasure. Source: ../en/admin-erasure.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-nav-erasure-coverage = 删除与清除
compliance-card-erasure-coverage = 在审计历史和日志中替换某个人的标识符，并查看哪些备份仍保存着已被删除的人员。
admin-erasure-title = 删除与清除
admin-erasure-intro = “清除人员”会用稳定的带密钥化名，替换审计历史和轮转日志文件中某人的地址和账户 ID。审计哈希链会重新封存，因此验证仍可通过。每次删除和清除还会留下一个墓碑，因此恢复较旧的备份时会再次删除这些人员。
admin-erasure-docs = 每张表中哪些内容被删除、导出和保留的登记表位于 docs/admin/erasure-coverage.md。
admin-erasure-scrub = 清除人员
admin-erasure-scrub-note = 请先运行预览：它会统计将发生变化的行数和日志行数，且不做任何更改。
admin-erasure-address = 地址
admin-erasure-preview = 预览
admin-erasure-run = 清除人员
admin-erasure-dry-note = 仅为预览。未做任何更改。
admin-erasure-done-note = 已清除。此人现在显示为 { $pseudonym }。
admin-erasure-nothing = 没有历史记录提到此人。
admin-erasure-col-table = 表或文件
admin-erasure-col-rows = 行数
admin-erasure-confirm-title = 从历史记录中清除此人？
admin-erasure-confirm-body = 提到此人的审计条目和日志行会用化名重写。此操作无法撤销；没有密钥就无法从化名恢复地址。
admin-erasure-settings = 设置
admin-erasure-keys = 化名密钥
admin-erasure-keys-note = 化名使用当前的密钥代数生成。轮换会增加一代；较旧的代数会保留，直到上方设置的保留期将其移除。
admin-erasure-rotate = 轮换密钥
admin-erasure-keys-none = 尚未创建密钥。首次清除时会创建一个。
admin-erasure-col-gen = 代数
admin-erasure-col-created = 创建时间
admin-erasure-col-state = 状态
admin-erasure-key-current = 当前
admin-erasure-key-retired = 已停用，保留
admin-erasure-key-destroy = 已停用，将于 { $when } 删除
admin-erasure-chain = 审计链
admin-erasure-chain-note = 每个审计条目都包含前一个条目的哈希。清除会重写条目并重新封存链，同时记录旧的和新的头部哈希。
admin-erasure-seal = 封存新条目
admin-erasure-chain-ok = 链验证通过：已封存 { $rows } 个条目，{ $unsealed } 个等待封存。
admin-erasure-chain-bad = 链验证未通过；第一个有问题的条目位于位置 { $seq }。
admin-erasure-col-time = 时间
admin-erasure-col-range = 位置
admin-erasure-backups = 备份
admin-erasure-backups-note = 备份在创建时会记录墓碑水位线。当前水位线为 { $watermark }。
admin-erasure-col-label = 标签
admin-erasure-col-location = 位置
admin-erasure-col-watermark = 水位线
admin-erasure-col-marker = 删除自
admin-erasure-col-actions = 操作
admin-erasure-backups-none = 没有登记任何备份。
admin-erasure-contains-erased = 包含已删除的主体（{ $count }）
admin-erasure-no-marker = 无
admin-erasure-forget = 忘记
admin-erasure-register = 登记备份
