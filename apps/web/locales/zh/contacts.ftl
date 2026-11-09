# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: contacts. Source: ../en/contacts.ftl

contacts-title = 联系人
contacts-search = 搜索联系人
contacts-new = 新建联系人
contacts-find-duplicates = 查找重复项
contacts-loading = 正在加载…
contacts-none = 没有联系人。
contacts-list = 联系人列表
contacts-favorite = 收藏 { $name }
contacts-export-vcard = 导出 vCard
contacts-export-csv = 导出 CSV
contacts-select-hint = 选择一个联系人以查看其名片。

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = 通讯录和群组
contacts-address-books = 通讯录
contacts-all = 所有联系人
contacts-favorites = 收藏
contacts-groups = 群组
contacts-new-group = 新建群组
contacts-new-group-name = 新群组名称
contacts-group-name-placeholder = 群组名称
contacts-create = 创建
contacts-data = 数据
contacts-import = 导入…

# -- Business card -----------------------------------------------------------
contacts-card = 联系人 { $name }
contacts-favorited = 已收藏
contacts-favorite-action = 收藏
contacts-email = 电子邮件
contacts-phone = 电话
contacts-dates = 日期
contacts-groups-label = 群组
contacts-notes = 备注
contacts-security-key = 安全密钥
contacts-key-on-file = 已存有密钥/证书（在支持 PGP/S-MIME 之前仅供显示）。
contacts-no-key = 未存有密钥。
contacts-directory-security = 目录安全信息
contacts-member-of = 所属群组
contacts-group-membership = { $name } 成员资格

# -- Editor ------------------------------------------------------------------
contacts-new-contact = 新建联系人
contacts-edit-contact = 编辑联系人
contacts-full-name = 全名
contacts-given = 名
contacts-given-placeholder = 名
contacts-surname = 姓
contacts-surname-placeholder = 姓
contacts-organization = 组织
contacts-job-title = 职位
contacts-job-title-placeholder = 职位
contacts-email-n = 电子邮件 { $n }
contacts-email-label = 电子邮件 { $n } 标签
contacts-email-label-placeholder = 工作
contacts-remove-email = 删除电子邮件 { $n }
contacts-add-email = 添加电子邮件
contacts-phone-n = 电话 { $n }
contacts-remove-phone = 删除电话 { $n }
contacts-add-phone = 添加电话
contacts-security-key-label = 安全密钥（不透明占位内容）
contacts-security-key-placeholder = PGP 密钥 / S-MIME 证书（仅供显示）
contacts-favorite-word = 收藏

# -- Import dialog -----------------------------------------------------------
contacts-import-title = 导入联系人
contacts-imported = 已导入 { $count ->
   *[other] { $count } 个联系人
}。
contacts-import-hint = 粘贴 vCard (.vcf) 或 CSV，或选择一个文件。格式会自动识别。
contacts-import-file = 导入文件
contacts-paste = 粘贴 vCard 或 CSV
contacts-preview = 预览
contacts-map-columns = 映射列
contacts-col = 列
contacts-maps-to = 映射到
contacts-sample = 示例
contacts-map-column = 映射列 { $header }
contacts-preview-count = 预览（{ $count }）
contacts-import-preview = 导入预览
contacts-no-name = （无名称）
contacts-no-email = 无电子邮件
contacts-import-n = 导入 { $count ->
   *[other] { $count } 个联系人
}

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = 合并重复项
contacts-merge-review-hint = 检查合并后的名片。其他名片将变为已删除标记（可撤销）。
contacts-merged-preview = 合并预览
contacts-emails = 电子邮件
contacts-phones = 电话
contacts-merge-confirm = 合并联系人
contacts-no-duplicates = 未找到重复项。
contacts-possible-duplicates = { $count } 个可能的重复项
contacts-review-merge = 检查合并

# -- Machine translation (2026-11): added to match en ---------------------
contacts-address-card = 联系人详情
contacts-address-recipients = 显示收件人
contacts-address-save = 保存联系人
contacts-address-compose = 发送邮件
contacts-address-unsaved = 此地址未保存在你的联系人中。
contacts-address-book = 通讯录
contacts-address-lookup-failed = 无法加载联系人信息，请重试。
contacts-address-save-failed = 无法保存联系人，请重试。
contacts-address-saved = 联系人已保存。
contacts-address-loading = 正在加载联系人…
contacts-editor-person = 姓名与工作
contacts-editor-reach = 邮箱与电话
contacts-editor-other = 备注与偏好
contacts-editor-save-error = 无法保存联系人。你的更改仍保留在此，请重试。
contacts-group-editor-help = 先在通讯录中创建群组，再从联系人卡片添加成员。
contacts-group-details = 群组详情
contacts-group-save-error = 无法保存群组。你的更改仍保留在此，请重试。
contacts-edit-group = 编辑联系人群组
contacts-edit-named-group = 编辑群组 { $name }
contacts-group-name = 群组名称
contacts-group-edit-help = 更新此群组的名称。其通讯录和成员保持不变。
contacts-back-to-list = 返回联系人
contacts-membership-title = 群组成员
contacts-membership-help = 选择一个群组，然后添加或移除成员。更改会即时保存。
contacts-membership-no-groups = 还没有群组。创建一个即可开始添加成员。
contacts-membership-pick = 选择一个群组以查看其成员。
contacts-membership-of = { $name } 的成员
contacts-membership-members = 成员
contacts-membership-add = 输入姓名或地址以添加成员
contacts-membership-add-group = 输入群组名称以添加此联系人
contacts-membership-failed = 部分成员更改未能保存。列表显示的是已保存的内容。
contacts-membership-manage = 管理群组成员
contacts-import-source = 来源
contacts-import-book = 通讯录
contacts-import-review = 检查
contacts-import-empty = 选择文件，或粘贴 vCard 或 CSV 文本，然后选择“预览”。找到的联系人会列在此处，你可以排除其中任何一项。
contacts-merge-sets = 可能的重复项
contacts-merge-set = { $name } 的重复项
contacts-merge-keep = 要保留的卡片
contacts-merge-done = 已合并到 { $name }。
contacts-address-actions = 联系人操作
contacts-address-save-to = 保存到联系人
contacts-address-goto = 转到联系人
contacts-address-copy = 复制联系人信息
contacts-address-copied = 联系人信息已复制。
contacts-address-copy-failed = 无法复制联系人信息。
contacts-share = 分享联系人
contacts-share-email = 通过邮件发送联系人
contacts-share-copy = 复制为 vCard
contacts-share-download = 下载 .vcf
contacts-share-native = 分享…
contacts-share-copied = vCard 已复制。
contacts-share-copy-failed = 无法复制 vCard。
contacts-share-downloaded = vCard 已下载。
contacts-share-failed = 无法分享该联系人。
contacts-share-attach-failed = 无法附加 .vcf 文件，因此 vCard 已放在邮件正文中。
contacts-address-saved-favorite = 联系人已保存并加入收藏。
contacts-favorite-add = 加入收藏
contacts-favorite-remove = 取消收藏
contacts-favorite-save = 保存并加入收藏
contacts-favorite-added = 已将 { $name } 加入收藏。
contacts-favorite-removed = 已将 { $name } 取消收藏。
contacts-favorite-failed = 无法更新收藏。
contacts-select-row = 选择 { $name }
contacts-selected-count = 已选 { $count } 项
contacts-select-clear = 清除选择
contacts-bulk-actions = 所选联系人的操作
contacts-bulk-favorites-added = 已将 { $count } 项加入收藏。
contacts-bulk-favorites-removed = 已将 { $count } 项取消收藏。
contacts-row-menu = { $name } 的操作
contacts-favorites-empty = 还没有收藏。使用联系人上的星标即可添加。

# -- Machine translation (2026-11): added to match en ---------------------
contacts-membership-count = { $count ->
   *[other] { $count } 位成员
}

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
contacts-save-already = { $name } 已在您的通讯录中。

contacts-save-already-badge = 已保存

contacts-save-already-many = 所有{ $count }联系人均已保存。

contacts-save-automated = 这看起来像一个自动发件人（无回复，通知）。

contacts-save-book = 地址簿

contacts-save-book-readonly = { $name }（只读）

contacts-save-cancel = 取消

contacts-save-choose = 选择联系人...

contacts-save-contacts = 保存此消息中的联系人

contacts-save-dialog-title = 保存此消息中的联系人

contacts-save-email = 电子邮件

contacts-save-failed = 无法保存联系人。再试一次。

contacts-save-group = 集团

contacts-save-group-none = 无组

contacts-save-loading = 正在加载地址...

contacts-save-name = 名称

contacts-save-new = 保存新联系人

contacts-save-no-book = 没有地址簿可以容纳新的联系人。

contacts-save-no-sender = 此消息没有发件人地址。

contacts-save-note = 从{ $date }的消息中保存

contacts-save-nothing = 没有人可以保存：每个人都已经在您的联系人中，是您，还是看起来是自动化的。

contacts-save-open = 打开

contacts-save-open-contact = 打开联系方式

contacts-save-open-link = 打开联系方式

contacts-save-own = 这是您自己的地址之一。

contacts-save-partial = 已保存 { $saved } 个联系人，共 { $total } 个。其他人无法获救。

contacts-save-pick = 节省{ $email }

contacts-save-recipients = 保存所有收件人

contacts-save-removed = 已保存的联系人已删除。

contacts-save-role-bcc = 密件抄送

contacts-save-role-cc = 抄送

contacts-save-role-from = 来自

contacts-save-role-replyTo = 回复

contacts-save-role-sender = 发件人

contacts-save-role-to = 至

contacts-save-saved-heading = 已在您的通讯录中

contacts-save-saved-many = 已将{ $count }联系人保存到{ $book }

contacts-save-saved-one = 已保存{ $name }至{ $book }

contacts-save-sender = 将发件人保存为联系人

contacts-save-settings-book = 地址簿

contacts-save-settings-book-default = 默认地址簿

contacts-save-settings-enabled = 优惠 保存此消息中的联系人

contacts-save-settings-enabled-help = 将项目添加到消息菜单和阅读器“更多”菜单。

contacts-save-settings-note = 添加带有日期的注释

contacts-save-settings-note-help = 注释上写着“从消息中保存”，后跟日期。

contacts-save-settings-reply-to = 对列表和通知发件人使用回复

contacts-save-settings-reply-to-help = 保存这些发件人的回复地址而不是发件人地址。

contacts-save-settings-scope = 在对话框中首先勾选

contacts-save-settings-scope-all = 大家都在留言

contacts-save-settings-scope-sender = 仅限发件人

contacts-save-settings-sender = 报价 将发件人保存为联系人

contacts-save-settings-sender-help = 消息菜单中的一键式项目。

contacts-save-settings-skip = 跳过自动发件人

contacts-save-settings-skip-help = 除非您勾选，否则将忽略无回复、通知和退回邮件地址。

contacts-save-settings-title = 保存消息中的联系人

contacts-save-show-automated = 显示自动地址和无回复地址

contacts-save-submit = { $count ->
       *[other] 保存 { $count } 个联系人
    }

contacts-save-undo = 撤消
