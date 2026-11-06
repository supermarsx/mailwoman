# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-role-purpose. Source: ../en/admin-role-purpose.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-role-purpose-super_admin = 对所有能力的完全管理，包括管理员、角色、安全、邮件路由和服务配置。请将此角色保留给负责整个部署的可信运维人员。
admin-role-purpose-domain_admin = 日常的用户和域名管理：创建、更新、禁用或删除用户；管理配额、用户能力、域名登录规则、设置和品牌；撤销会话和管理封禁。还包括邮件诊断和使用情况/数据导出。不包括管理员角色管理或上游服务器更改。
admin-role-purpose-helpdesk = 账户支持和事件分流：查看用户、审计活动和使用情况；更改用户标志、禁用账户和撤销会话；查看封禁并运行登录模拟。不包括创建用户、删除、配额更改或一般设置写入。
admin-role-purpose-auditor = 检查和报告：授予所有以 .read 结尾的能力，以及审计和使用情况导出。不包括配置更改或账户变更。导出的报告可能包含敏感的管理信息。
admin-role-purpose-security_officer = 安全运营：管理安全和 MFA 策略、登录策略和封禁；禁用用户、撤销会话或 API 密钥；查看并导出审计活动；读取认证日志并运行诊断。SSO 配置可读取，但此角色不包括 SSO 编辑或管理员角色管理。
admin-role-purpose-custom = 一个包含 { $count } 项所选权限的自定义角色。其权限取决于下方的确切权限列表，而不是其名称。分配之前请审阅每项操作。
admin-role-purpose-scope = 域名范围在将此角色授予管理员时分配。账户级的允许/拒绝覆盖可能改变实际访问权限。描述本身不会授予额外权限。
admin-role-purpose-editor = 选择此管理员需要执行的任务，然后选中相应的权限。读取访问、导出数据和更改记录是各自独立的操作。保存之前请审阅确切的能力标识符。
admin-role-purpose-selection = 这些说明解释所选的操作。它们不会扩大发送给服务器的权限。
admin-role-purpose-read = 查看此区域中的信息。
admin-role-purpose-export = 从此区域导出信息。
admin-role-purpose-revoke = 撤销此权限所指明的会话或凭据。
admin-role-purpose-delete = 删除此权限所涵盖的记录。
admin-role-purpose-disable = 禁用此权限所涵盖的账户。
admin-role-purpose-create = 创建此权限所涵盖的记录。
admin-role-purpose-specific = 执行此能力所指明的特定操作；授予之前请审阅其标识符。
admin-role-purpose-description = 目的和职责
admin-role-purpose-description-help = 说明谁应该获得此自定义角色以及他们处理的任务（最多 1024 个字符）。此文本不会更改其权限。
