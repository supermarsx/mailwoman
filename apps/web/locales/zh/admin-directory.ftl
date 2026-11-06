# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-directory. Source: ../en/admin-directory.ftl


# -- Machine translation (2026-11): added to match en ---------------------
directory-title = LDAP 目录配置
directory-help = 配置目录服务器、属性映射和加密保存的服务凭据。更改在服务器重启后生效。此处不会测试服务器的可达性。
directory-reload = 重新加载已保存的配置
directory-saved = 配置已保存。请重启服务器以应用。
directory-restart = 已保存的配置与正在运行的目录不同。需要重启服务器。
directory-proxy-blocked = 出站代理处于活动状态时，LDAP 连接会被阻止。此 LDAP 传输不支持代理隧道，也不会绕过你的代理。
directory-env-pin = 环境变量已固定此绑定 DN 的服务凭据：
directory-url = 服务器 URL
directory-base = 搜索基础 DN
directory-bind = 服务绑定 DN
directory-tls = 传输安全
directory-plaintext = 明文（仅限匿名）
directory-priority = 优先级
directory-connection = 连接策略
directory-blocked = 阻止连接
directory-direct = 允许受保护的直接连接
directory-enabled = 包含在目录搜索中
directory-private = 允许私有局域网和回环地址
directory-consent = 连接已被阻止。请明确选择受保护的直接访问以允许此端点；私有地址需要另行同意。
directory-password-set = 已存储加密的服务密码。
directory-password-unset = 未存储服务密码。除非适用环境变量凭据，否则搜索使用匿名访问。
directory-secret-action = 已存储密码的操作
directory-keep = 保持不变
directory-replace = 设置或替换密码
directory-clear = 清除已存储的密码
directory-password = 新的服务密码
directory-attr-display-name = 显示名称属性
directory-attr-mail = 电子邮件属性
directory-attr-member = 组成员属性
directory-attr-user-cert = 证书属性
directory-attr-photo = 照片属性
directory-remove = 从配置中移除
directory-add = 添加目录
directory-save = 保存目录配置
