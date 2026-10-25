# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: passwd. Source: ../en/passwd.ftl

# -- Region / heading --------------------------------------------------------
passwd-region-label = 更改密码
passwd-heading = 更改密码

# -- Forced change -----------------------------------------------------------
passwd-force-change = 管理员要求你先更改密码才能继续。

# -- Fields ------------------------------------------------------------------
passwd-current-label = 当前密码
passwd-new-label = 新密码
passwd-confirm-label = 确认新密码

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = 两次输入的新密码一致。
passwd-match-no = 两次输入的新密码尚不一致。

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = 至少 { $count } 个字符
passwd-rule-uppercase = 一个大写字母
passwd-rule-lowercase = 一个小写字母
passwd-rule-digit = 一个数字
passwd-rule-symbol = 一个符号

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = 此账户为零访问账户。在应用更改之前，会向你显示一个恢复短语——请保存好它，以便在出现任何问题时仍能访问你的数据。

# -- Actions -----------------------------------------------------------------
passwd-continue = 继续
passwd-submit = 更改密码

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = 保存恢复短语
passwd-recovery-prose = 请写下此短语并存放在安全的地方。它在更改密码之前显示，这样即使新密码丢失，你也能恢复数据。它不会保存在服务器上。
passwd-recovery-ack-label = 我已保存恢复短语
passwd-recovery-ack-text = 我已将恢复短语保存在安全的地方。

# -- Done phase --------------------------------------------------------------
passwd-done = 密码已更改。
passwd-done-reencrypt = 已保存的服务器凭据已使用新密码重新加密。
passwd-done-rewrap = 零访问密钥已使用新密码重新封装。

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = 请输入当前密码
passwd-error-mismatch = 新密码与确认密码不一致
passwd-error-policy = 新密码需要包含{ $rules }
passwd-error-ack-first = 请先确认已保存恢复短语
passwd-error-prepare-recovery = 无法准备恢复短语
passwd-error-change = 无法更改密码
# Shown under the server's own message when a change is refused. Each line states
# what the status means; none of them claims the password was or was not changed.
passwd-error-session-ended = 会话已结束。请重新登录。
passwd-error-not-configured = 此服务器未设置密码更改功能。请联系管理员。
passwd-error-contact-admin = 服务器无法完成更改。请联系管理员。

# -- Forced change screen ----------------------------------------------------
# Shown instead of the mailbox while an administrator's "force password change"
# flag is set on the account. The server refuses mail requests until the change
# succeeds, so this copy must say exactly that — including the case where the
# server has no way to change the password.
passwd-forced-title = 请更改密码以继续
passwd-forced-signed-in-as = 已登录为 { $username }。
passwd-forced-explain = 管理员要求此账户更改密码。在更改成功之前，邮件、日历和联系人均不可用。
passwd-forced-backend-note = 如果当前密码正确但更改仍被拒绝，则此服务器可能无法更改你的密码。请联系管理员：管理员可以设置密码更改功能或取消此要求。
passwd-forced-sign-out = 退出登录
passwd-forced-still-required = 服务器已接受更改，但仍要求此账户更改密码。请联系管理员。
passwd-forced-reload-failed = 密码已更改，但无法加载你的账户。请检查网络连接后重试。
passwd-forced-retry = 重试
