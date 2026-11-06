# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-identity. Source: ../en/admin-identity.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-identity-title = 身份提供方
admin-identity-description = 允许经批准的应用使用 Mailwoman 登录。这与外部登录提供方和邮件 API 访问是分开的。必须使用带 PKCE 的授权码；其他授权类型不可用。
admin-identity-configure = 配置颁发者
admin-identity-new = 添加身份客户端
admin-identity-edit = 编辑客户端
admin-identity-enabled = 已启用
admin-identity-disabled = 已禁用
admin-identity-all = 全部
admin-identity-discovery = 打开发现文档
admin-identity-search = 搜索客户端
admin-identity-status = 状态
admin-identity-clients = 身份客户端
admin-identity-client = 客户端身份
admin-identity-client-id = 客户端 ID
admin-identity-name = 名称
admin-identity-method = 客户端认证
admin-identity-public = 公共客户端（PKCE，无密钥）
admin-identity-pkce = 每个客户端都必须使用 PKCE S256。机密客户端的密钥在此生成，且只显示一次。
admin-identity-rotate-secret = 替换现有的客户端密钥
admin-identity-redirects = 已注册的目标地址
admin-identity-redirect-help = 每行一个精确的 HTTPS URI；本地应用允许使用数字回环 HTTP。通配符和片段会被拒绝。
admin-identity-callbacks = 授权回调 URI
admin-identity-logout-uris = 登出后重定向 URI
admin-identity-scopes = 允许的身份范围
admin-identity-actions = 操作
admin-identity-delete = 删除客户端
admin-identity-delete-confirm = 删除 { $name } 的身份访问权限？现有的身份授权将失效，其仅用于身份的客户端注册也将被移除。
admin-identity-empty = 没有匹配的身份客户端。
admin-identity-cancel = 取消
admin-identity-save = 保存
admin-identity-done = 完成
admin-identity-secret = 生成的客户端密钥
admin-identity-secret-once = 请立即将此密钥复制到依赖方应用中。它无法再次显示。切勿把机密密钥放进浏览器代码。
admin-identity-issuer = 公共 HTTPS 颁发者 URL
admin-identity-issuer-help = 使用此部署的精确 HTTPS 源和 /oidc 路径，包括任何部署前缀。更改颁发者或配置会撤销现有的身份授权。
admin-identity-rotate-key = 轮换签名密钥
admin-identity-key-help = 轮换需要重新进行管理员验证。旧的公钥会保留 15 分钟，以便最近颁发的令牌能完成其较短的有效期。
