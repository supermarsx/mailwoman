# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: auth. Source: ../en/auth.ftl

# -- Mailbox login -----------------------------------------------------------
# The sign-in screen asks for the email address and the password only. The
# server decides which mail server the address signs in to; the user never
# enters or sees one.
auth-app-name = Mailwoman
auth-email = 电子邮件地址
auth-password = 密码
auth-sign-in = 登录
auth-signing-in = 正在登录…
auth-invalid-credentials = 凭据无效
# Shown under the refusal. The server answers a disabled account with the same
# response as a wrong password, so this states a possibility, not a diagnosis.
auth-refused-note = 如果电子邮件地址和密码正确，该账户可能已被管理员停用。已停用的账户在管理员重新启用之前无法登录。
auth-unreachable = 无法连接到服务器
# The server has no mail server configured for the address's domain.
auth-address-not-configured = 此服务器上未设置该地址。请联系管理员。

# -- Single sign-on (t9) -----------------------------------------------------
# The "or continue with" divider + one button per configured IdP. Only shown
# when the deployment has enabled SSO backends; otherwise the login is unchanged.
auth-sso-heading = 单点登录
auth-sso-divider = 或通过以下方式继续
# `name` is the IdP's admin-set (trusted-operator) display name.
auth-sso-button = 使用 { $name } 登录
# Shown when an SSO round-trip fails and the IdP returns to /?sso_error — a
# uniform message that never reveals which check failed (no-leak, like the 401).
auth-sso-error = 单点登录未完成。请重试，或使用密码登录。

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = 授权应用
auth-consent-title = 授权访问
auth-consent-loading = 正在加载请求…
# Rendered after the (isolated) client-name span: "<client> wants to access…".
auth-consent-intro = 请求访问你的账户。
auth-consent-approved = 经管理员批准的客户端
auth-consent-unapproved = 无法识别的客户端 — 未经管理员批准
auth-consent-requesting = 请求的权限
auth-consent-redirects-to = 重定向到
auth-consent-for-resource = 用于资源
auth-consent-deny = 拒绝
auth-consent-allow = 允许
auth-consent-error = 无法记录你的决定
