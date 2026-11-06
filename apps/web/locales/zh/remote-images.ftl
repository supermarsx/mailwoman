# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = 远程内容

# -- Machine translation (2026-11): added to match en ---------------------
remote-change-failed = 无法更新远程图片权限。{ $reason }
remote-banner-blocked = 已阻止远程图片
remote-banner-blocked-policy = 已根据你的隐私设置阻止远程图片
remote-banner-shown = 已显示图片
remote-banner-proxied = 图片已通过服务器加载
remote-banner-direct = 图片已从发件人的服务器加载
remote-banner-trackers = { $count ->
   *[other] 已移除 { $count } 个跟踪器
}
remote-banner-trackers-limited = 至少已移除 { $count } 个跟踪器
remote-banner-show = 显示图片
remote-banner-show-help = 通过服务器加载此邮件的图片，这样发件人看不到你的 IP 地址
remote-banner-more = 更多图片选项
remote-banner-always-sender = 始终显示来自 { $sender } 的图片
remote-banner-always-domain = 始终显示来自 { $domain } 的图片
remote-banner-always-all = 在所有邮件中始终显示图片
remote-banner-settings = 隐私设置
remote-banner-undo = 撤销
remote-banner-undo-help-single = 再次阻止此邮件的图片
remote-banner-undo-help-per-sender = 不再自动显示来自 { $sender } 的图片
remote-banner-undo-help-per-domain = 不再自动显示来自 { $domain } 的图片
remote-banner-undo-help-all = 不再自动在所有邮件中显示图片
remote-banner-blocked-help = 存储在其他服务器上的图片可以告知发件人你何时何地打开了此邮件。在你选择显示之前，它们会一直被阻止。
remote-banner-help-block = 你的隐私设置会阻止所有远程图片。可在隐私设置中更改。
remote-banner-help-ask = 图片通过此服务器加载，这样发件人看不到你的 IP 地址。
remote-banner-help-proxy = 图片通过此服务器加载，这样发件人看不到你的 IP 地址。
remote-banner-help-direct = 图片直接从发件人的服务器加载，对方可以看到你的 IP 地址。
remote-banner-hosts = 已阻止的主机：{ $hosts }
remote-banner-trackers-help = 跟踪图片会在邮件显示前被移除。服务器仅在下载后才识别为跟踪器的图片，会在有限时间内计入统计。
remote-images-toast-loading = 正在加载图片…
remote-images-toast-done = 图片已加载
remote-images-toast-failed = { $count ->
   *[other] { $count } 张图片无法加载
}
remote-images-toast-retry = 重试
remote-images-toast-progress = 正在加载图片… { $loaded }/{ $total }
remote-images-toast-retrying-progress = 正在加载图片… { $loaded }/{ $total } · 第 { $attempt }/{ $max } 次重试
remote-images-toast-cancel = 取消
remote-images-toast-cancelled = 已取消加载图片
remote-images-toast-load-again = 重新加载
remote-banner-hosts-manage = 图片来源…
image-hosts-title = 按来源管理图片
image-hosts-help = 为每个来源的图片选择处理方式。“从不显示”优先于“始终显示”，并适用于所有邮件。
image-hosts-none = 此邮件没有远程图片。
image-hosts-for = 来自 { $host } 的图片
image-hosts-allow = 始终显示
image-hosts-ask = 询问
image-hosts-block = 从不显示
image-hosts-failed = 无法保存图片来源设置。
image-tip-blocked = 图片已被阻止。选择“显示图片”即可加载。
image-tip-loading = 正在加载图片…
image-tip-error = 图片不可用
