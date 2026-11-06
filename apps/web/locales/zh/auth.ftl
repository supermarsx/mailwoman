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

# -- Machine translation (2026-11): added to match en ---------------------
auth-login-rate-limited = 登录尝试次数过多，请稍后再试。
auth-retry-countdown = { $seconds } 秒后可重试
auth-upstream-protocol = 登录已中止：邮件服务器返回了此服务器无法解析的响应。请稍后重试，或告知管理员。
auth-account-disabled = 此账户已被管理员停用，在管理员重新启用之前无法登录。
auth-signin-not-allowed = 此地址不能登录本服务器。请联系管理员。
auth-signin-not-provisioned = 此地址尚无账户，且不会自动创建新账户。请联系管理员为您添加。
auth-reference = 参考编号：{ $id }
auth-domain-hint = 对于 { $domain }，只需使用用户名即可登录。
auth-passkey-sign-in = 使用通行密钥登录
auth-passkey-failed = 无法使用通行密钥登录。请重试或使用密码。
auth-remembered = 以前的登录
auth-remembered-help = 在此设备上成功使用过的地址。选择后仅填入地址。
auth-forget-address = 在此设备上忘记 { $address }
auth-login-timeout = 登录超时，未进行自动重试。请检查网络连接，准备好后再试。
auth-verification-title = 登录验证
auth-verification-loading = 正在加载登录验证…
auth-verification-error = 无法完成验证。登录前请重新进行验证。
auth-verification-required = 请先完成新的登录验证，然后再重试。
auth-identity-title = 共享您的身份
auth-identity-sign-in = 请重新登录以继续此身份请求。
auth-identity-sign-out = 退出 Mailwoman
auth-identity-expired = 此身份请求已过期或已更改。请返回发起请求的应用并重新开始。
auth-identity-retry = 重试
auth-identity-disclosure = 此应用将收到以下身份信息。此授权不会授予访问您邮件的权限。
auth-identity-scope-openid = 稳定的身份标识符
auth-identity-scope-profile = 可用的个人资料信息
auth-identity-scope-email = 您登录时使用的邮箱地址（不保证已独立验证）
auth-identity-scope-offline_access = 在此 Mailwoman 会话有效期间续期身份访问，自登录起最长 15 天
auth-identity-deny = 取消
auth-identity-approve = 允许身份访问
auth-identity-grants = 使用您身份的应用
auth-identity-grants-help = 撤销某个应用的身份访问权限。这不会移除单独的邮件 API 权限。
auth-identity-no-grants = 目前没有应用获得身份授权。
auth-identity-revoke = 撤销身份访问
auth-maintenance = 网页邮箱正在维护，暂时停用。请稍后再试。
