# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = 远程内容

# -- Blocked state: count summary -------------------------------------------
remote-blocked-count = { $count ->
   *[other] 已阻止 { $count } 张远程图片
}
remote-blocked-trackers = { $blocked ->
   *[other] { $blocked } 张远程图片中
}有 { $count ->
   *[other] { $count } 个跟踪器被阻止
}
remote-blocked-hosts = 被阻止的主机

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = 加载图片
remote-grant-sender = 始终加载来自 { $sender } 的图片
remote-grant-domain = 始终加载来自 { $domain } 的图片
remote-grant-all = 始终加载所有远程图片

remote-grant-done-single = 已为此邮件加载远程图片。
remote-grant-done-per-sender = 将加载来自此发件人的远程图片。
remote-grant-done-per-domain = 将加载来自此域名的远程图片。
remote-grant-done-all = 将为所有邮件加载远程图片。

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = 正在为此邮件加载远程图片。
remote-revoke = 关闭
remote-revoke-done = 已关闭远程图片。下次打开时将再次阻止。
