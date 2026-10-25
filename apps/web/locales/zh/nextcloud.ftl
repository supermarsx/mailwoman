# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = 上一级
nextcloud-loading = 正在加载…
nextcloud-list-error = 无法列出此文件夹的内容。
nextcloud-empty = 此文件夹为空。
nextcloud-file-list = Nextcloud 文件
nextcloud-open-folder = 打开文件夹 { $name }
nextcloud-select-file = 选择 { $name }

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = 从 Nextcloud 添加附件
nextcloud-attach-action = { $count ->
    [0] 添加附件
   *[other] 添加 { $count } 个文件为附件
  }
nextcloud-error-select-file = 请至少选择一个要添加为附件的文件
nextcloud-error-attach-failed = 无法将所选文件添加为附件

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = 保存到 Nextcloud
nextcloud-save-title = 将“{ $name }”保存到 Nextcloud
nextcloud-save-action = 保存到此处（{ $dir }）
nextcloud-saved = 已保存到 { $path }
nextcloud-error-save-failed = 无法保存到 Nextcloud

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = 创建共享链接
nextcloud-share-title = 共享链接
nextcloud-share-intro = 创建指向 { $path } 的公开链接，而不是将文件作为附件。
nextcloud-protect-password = 使用密码保护
nextcloud-password-label = 密码
nextcloud-share-password = 共享密码
nextcloud-set-expiry = 设置到期日期
nextcloud-expires-on = 到期日
nextcloud-expiry-date = 到期日期
nextcloud-create-link = 创建链接
nextcloud-password-protected = 受密码保护
nextcloud-no-password = 无密码
nextcloud-expires = { $date } 到期
nextcloud-no-expiry = 无到期时间
nextcloud-error-need-password = 请输入密码，或关闭密码保护
nextcloud-error-need-expiry = 请选择到期日期，或关闭到期设置
nextcloud-error-share-failed = 无法创建共享链接
