# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: plugins. Source: ../en/plugins.ftl

# -- Unsigned-plugin trust banner --------------------------------------------
plugins-unsigned-warning-label = 未签名 UI 插件警告
plugins-unsigned-title = 未签名的插件正在运行
plugins-unsigned-text = 这些插件在没有经过验证的签名的情况下被允许加载。它们在沙箱中运行，但其代码尚未通过受信任密钥的校验。

# -- Plugin frames: the region around each sandboxed frame and the frame title -
plugins-frame-label = 插件：{ $name }
plugins-frame-title = 插件：{ $id }

# -- Admin: engine plugins (src/screens/Admin/Plugins) ------------------------
plugins-admin-intro = 引擎插件在 WebAssembly 沙箱中运行。插件只有在注册、批准、至少授予一项能力并启用之后才会运行。它只能使用此处授予的能力。
plugins-admin-unsigned-banner = 已加载一个没有签名的第三方插件。它之所以运行，是因为管理员允许了它；服务器仅根据已批准的摘要校验其文件。
plugins-admin-load-error = 无法加载插件。
plugins-admin-register-heading = 注册插件
plugins-admin-register-component = 组件
plugins-admin-register-third-party = 第三方组件
plugins-admin-register-id = 插件 ID
plugins-admin-register-id-hint = 服务器会从第三方插件目录中以此 ID 命名的文件读取组件。请先在下方的允许列表中批准该文件的摘要。
plugins-admin-register-name = 名称
plugins-admin-register-version = 版本
plugins-admin-register-capabilities = 插件声明的能力
plugins-admin-register-hosts = 插件可以连接的主机
plugins-admin-register-hosts-hint-first-party = 以逗号分隔的主机名。留空则使用组件的默认主机。
plugins-admin-register-hosts-hint-third-party = 以逗号分隔的主机名。留空则不允许访问网络。
plugins-admin-register-submit = 注册
plugins-admin-trust-first-party = 第一方，摘要内置
plugins-admin-trust-third-party = 第三方，摘要已批准
plugins-admin-status-loaded = 已加载
plugins-admin-status-not-loaded = 未加载
plugins-admin-status-restart = 需要重启
plugins-admin-restart-to-load = 需要重启才能加载此插件。
plugins-admin-restart-to-apply = 需要重启才能应用这些更改。在此之前，此桥接的部分账户不会得到服务。
plugins-admin-reason-not-approved = 未加载：管理员尚未批准此插件。
plugins-admin-reason-disabled = 未加载：插件未启用。
plugins-admin-reason-unsigned-not-allowed = 未加载：插件没有签名，且未被允许在无签名的情况下运行。
plugins-admin-reason-no-grant = 未加载：尚未授予其所需的任何能力。
plugins-admin-reason-component-unavailable = 未加载：没有组件文件通过摘要校验。
plugins-admin-reason-load-failed = 未加载：服务器无法加载该组件。请查看服务器日志。
plugins-admin-reason-proxy-mode = 未加载：此服务器以代理模式运行，自身不处理邮件。
plugins-admin-reason-no-account-binding = 未加载：没有账户绑定到此桥接。
plugins-admin-reason-no-host-caller = 未加载：此服务器版本不会调用此插件提供的钩子。
plugins-admin-reason-another-classifier-active = 未加载：已加载另一个垃圾邮件分类器。同一时间只能运行一个。
plugins-admin-running-with = 运行能力：{ $capabilities }
plugins-admin-hosts = 主机：{ $hosts }
plugins-admin-hosts-none = 主机：无
plugins-admin-grants-heading = 已授予的能力
plugins-admin-grants-hint = 插件只能使用此处勾选的能力。保存后会替换之前的授权。
plugins-admin-grant-for = 向 { $name } 授予 { $capability }
plugins-admin-grants-save = 保存授权
plugins-admin-endpoint = 分类器地址
plugins-admin-endpoint-for = { $name } 的分类器地址
plugins-admin-endpoint-hint = 插件连接其守护进程的地址。该主机也必须在插件的主机列表中。留空则使用组件的默认值。
plugins-admin-endpoint-save = 保存地址
plugins-admin-test = 测试分类器
plugins-admin-test-verdict = 测试结果：{ $verdict }
plugins-admin-uninstall = 卸载
plugins-admin-uninstall-confirm = 确认卸载
plugins-admin-uninstall-note = 卸载会移除注册信息、其授权及其保存的设置。
plugins-admin-uninstall-detail = 卸载会删除该插件为所有账户保存的数据、其允许列表固定项，以及其注册信息和授权。磁盘上的组件文件不会被删除。
plugins-admin-revoke-detail = 吊销会撤销对此摘要的批准并禁用插件。如果插件正在运行，会立即停止。
plugins-admin-error-already-registered = 此插件已注册。请先卸载，再重新注册。
plugins-admin-error-component-unavailable = 此插件没有与服务器内置摘要相匹配的组件文件。
plugins-admin-error-digest-not-approved = 未找到此 ID 对应的、摘要已获批准的组件文件。请先在允许列表中批准该文件的摘要。
plugins-admin-error-not-approved = 请先批准插件，再启用它。
plugins-admin-error-unsigned-not-allowed = 此插件没有签名。请先允许它在无签名的情况下运行，再启用它。
plugins-admin-error-not-loaded = 插件未加载，因此无法测试。
plugins-admin-error-classifier-error = 分类器调用失败：{ $detail }
plugins-admin-error-other = 服务器拒绝了请求：{ $detail }

# -- Admin: UI plugins (src/screens/Admin/UiPlugins) --------------------------
plugins-admin-ui-nav = UI 插件
plugins-admin-ui-title = UI 插件
plugins-admin-ui-intro = UI 插件为网页客户端添加控件。每个插件都在沙箱框架中运行，只能使用此处授予的能力。插件在批准并启用后，会提供给已登录的用户。
plugins-admin-ui-empty = 没有已注册的 UI 插件。
plugins-admin-ui-load-error = 无法加载 UI 插件。
plugins-admin-ui-register-heading = 注册 UI 插件
plugins-admin-ui-manifest = 清单（JSON）
plugins-admin-ui-bundle = 包文件
plugins-admin-ui-bundle-hint = 签名插件必须提供：签名将根据此文件进行校验。
plugins-admin-ui-allow-unsigned = 允许此插件在无签名的情况下注册
plugins-admin-ui-register = 注册
plugins-admin-ui-manifest-invalid = 清单不是有效的 JSON。
plugins-admin-ui-refused = 服务器拒绝了请求：{ $detail }
plugins-admin-ui-approve = 批准并启用
plugins-admin-ui-extension-points = 扩展点：{ $points }
plugins-admin-ui-grants-heading = 授予能力
plugins-admin-ui-grants-note = 服务器不会报告哪些能力已被授予。再次授予某项能力会替换其之前的授权。
plugins-admin-ui-grant = 授予 { $capability }
plugins-admin-ui-grant-for = 向 { $name } 授予 { $capability }
plugins-admin-ui-grant-hosts = { $name } 上 { $capability } 的主机
plugins-admin-ui-grant-hosts-hint = 插件可以从中获取内容的主机名，以逗号分隔。
plugins-admin-ui-granted = 已授予 { $capability }。
plugins-admin-ui-delete = 删除
plugins-admin-ui-delete-confirm = 确认删除
