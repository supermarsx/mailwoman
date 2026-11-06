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

# -- Machine translation (2026-11): added to match en ---------------------
nextcloud-share-upload-intro = { $path } 超过了附件大小限制。请将其上传到你 Nextcloud 的“Mail attachments”文件夹，并改为在邮件中插入共享链接。
nextcloud-upload-and-link = 上传并插入链接
nextcloud-share-not-now = 暂不
nextcloud-upload-progress = 正在上传：{ $percent }%
nextcloud-upload-progress-label = 上传进度
nextcloud-share-inserted = 已在邮件中插入 { $name } 的共享链接。
nextcloud-share-body = { $name }（{ $size }）：{ $url }
nextcloud-share-body-expires = 该链接在 { $date } 之前有效。
nextcloud-share-body-password = 该链接受密码保护。
nextcloud-account-title = Nextcloud 账户
nextcloud-account-help = 将你自己的账户关联到管理员批准的实例。凭据仅为此邮箱账户加密存储。
nextcloud-account-load-error = 无法加载 Nextcloud 关联。请重新加载后重试。
nextcloud-account-unlink-error = 无法移除关联。请重新加载后再试。
nextcloud-account-reload = 重新加载
nextcloud-account-no-instances = 没有可用的已启用 Nextcloud 实例。请联系管理员配置。
nextcloud-account-ready = 已关联，可使用
nextcloud-account-stale = 实例配置已更改。请重新验证关联。
nextcloud-account-unlinked = 未关联
nextcloud-account-relink = 重新验证关联
nextcloud-account-link = 关联账户
nextcloud-account-unlink = 取消关联账户
nextcloud-account-password-help = 在 Nextcloud 的个人安全设置中创建应用密码，并将其用户名和密码复制到这里。验证只会发出一次有限的请求；被拒绝的凭据不会重试。
nextcloud-account-username = Nextcloud 应用密码用户名
nextcloud-account-password = Nextcloud 应用密码
nextcloud-account-verify = 验证并关联
nextcloud-account-verify-error = 验证或保存失败。请检查实例、应用密码用户名和密码。密码字段已清空；现有可用的关联未被替换。
nextcloud-admin-title = Nextcloud 实例
nextcloud-admin-help = 批准邮箱用户可以各自关联的 HTTPS 实例。邮箱账户不会获得共享的服务密码。文件操作限制为 8 MiB。使用原生引擎的用户在关联后需明确发现并连接日历、任务和通讯录集合。
nextcloud-admin-add = 添加 Nextcloud 实例
nextcloud-admin-edit = 编辑 Nextcloud 实例
nextcloud-admin-enabled = 已启用
nextcloud-admin-disabled = 已禁用
nextcloud-admin-missing = 此实例已不存在。请返回实例列表。
nextcloud-admin-private-help = 私有地址需要明确的 Nextcloud 出站权限。已配置的代理路由仍具有权威性；被拒绝的代理请求绝不会回退为直接连接。
nextcloud-admin-revalidation = 保存任何实例更改都需要用户重新验证其关联。禁用实例会立即停止新的操作。
nextcloud-admin-authority = 实例权限
nextcloud-admin-name = 显示名称
nextcloud-admin-name-help = 用户选择实例时显示的简短名称。
nextcloud-admin-url = HTTPS 安装 URL
nextcloud-admin-url-help = 使用安装根地址，例如 https://cloud.example.org/ 或 https://example.org/nextcloud/。证书必须受信任并与主机名匹配。不接受凭据、查询参数和片段。
nextcloud-admin-enable = 允许用户关联此实例
nextcloud-admin-ca = 其他证书颁发机构（PEM）
nextcloud-admin-ca-help = 此 Nextcloud 服务器可选的私有 CA 证书。公共信任仍然可用，并且仍需验证主机名。更改后需要用户重新验证其关联；代理证书信任是独立的。
nextcloud-instance-select = Nextcloud 实例
nextcloud-instance-placeholder = 选择已关联的实例
nextcloud-instances-empty = 没有已验证的 Nextcloud 关联。请在“用户设置 → Nextcloud”中添加或验证关联。
nextcloud-instances-error = 无法加载已关联的实例。请重新打开选择器重试。
nextcloud-save-warning = 管理员要求在此附件离开邮箱之前先确认。
nextcloud-save-acknowledge = 我了解附件警告，并希望将此文件保存到所选的 Nextcloud 实例。
nextcloud-save-attachment = 将 { $name } 保存到 Nextcloud
nextcloud-save-close = 关闭 Nextcloud 目标位置
nextcloud-save-options = 将附件保存到 Nextcloud
nextcloud-pim-title = 联系人和日历
nextcloud-pim-help = 从此已验证的 Nextcloud 账户发现集合，然后明确连接你需要的集合。同步可拉取最新数据；已连接集合的编辑使用远程集合，如果其关联或权限被撤销则会失败。
nextcloud-pim-category = 集合类别
nextcloud-pim-calendar = 日历
nextcloud-pim-contacts = 通讯录
nextcloud-pim-tasks = 任务列表
nextcloud-pim-discover = 发现集合
nextcloud-pim-discovered = 可用集合
nextcloud-pim-empty = 未返回匹配的集合。
nextcloud-pim-connect = 连接
nextcloud-pim-connected = 已连接的集合
nextcloud-pim-sync = 立即同步
nextcloud-pim-sync-needed = 集合已连接。请立即同步以加载其内容。
nextcloud-pim-synced = 同步已完成。
nextcloud-pim-error = 操作失败。请检查账户关联和权限，然后明确重试。现有缓存数据未被丢弃。
nextcloud-pim-native-required = 联系人和日历同步需要原生邮件引擎。文件访问仍可通过你的账户关联使用。
nextcloud-pim-disconnect = 断开连接
nextcloud-pim-disconnect-help = 断开连接会保留缓存数据并停止向远程写入。重新关联的账户需要再次发现并明确连接。
nextcloud-pim-ready = 就绪
nextcloud-pim-disconnected = 已断开或关联已更改
nextcloud-pim-unnamed = 未命名的集合
nextcloud-pim-back = 返回 Nextcloud 账户
nextcloud-pim-create-title = 在 Nextcloud 中新建集合
nextcloud-pim-create-help = 在此 Nextcloud 账户中创建一个空的日历或任务列表，并在此处连接它。
nextcloud-pim-create-name = 新集合的名称
nextcloud-pim-create = 在 Nextcloud 中创建
nextcloud-pim-created = { $name } 已在 Nextcloud 中创建并连接。请立即同步以加载。
nextcloud-pim-interval = 自动刷新间隔（秒）
nextcloud-pim-interval-help = 0 表示仅手动刷新。正数间隔最短为 60 秒。已连接的账户会在后台刷新；失败时会退避，凭据被拒绝后将停止后续请求，直到你重新关联。
nextcloud-pim-interval-reset = 使用管理员默认值
nextcloud-admin-empty = 尚无已批准的实例。添加实例后才可用于账户关联。
nextcloud-admin-status = 状态
nextcloud-admin-open = 打开 { $name }
