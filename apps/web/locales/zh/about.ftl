# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: about. Source: ../en/about.ftl

about-tp-label = 第三方库和许可证
about-tp-search = 按名称、版本或许可证搜索
about-tp-filter = 按许可证筛选
about-tp-filter-all = 全部（{ $n }）
about-tp-filter-licence = { $licence }（{ $n }）
about-tp-showing =
    { $count ->
        [0] 没有匹配的条目。
       *[other] 显示 { $total } 个条目中的 { $shown } 个。
    }
about-tp-empty = 没有符合此搜索和许可证的条目。
about-tp-clear = 清除搜索和筛选
about-tp-more = 显示更多（剩余 { $n } 个）

about-tp-kind-crate = Rust crate
about-tp-kind-npm = npm 包
about-tp-kind-font = 字体
about-tp-kind-icons = 图标
about-tp-kind-image = 图片
about-tp-kind-os = 镜像软件包

about-tp-group-count = { $title }（{ $n }）
about-tp-group-rust = 服务器、Web 应用和插件中的 Rust crate
about-tp-group-shells = 仅用于桌面和移动应用的 Rust crate
about-tp-group-web = Web 应用中的 npm 包
about-tp-group-assets = 字体、图标和图片
about-tp-group-image = 服务器镜像文件
about-tp-group-build = 构建工具：构建时运行，不随产品分发

about-tp-licence = 许可证：
about-tp-licence-none = 未声明
about-tp-licence-in-file = 见版权文件
about-tp-used-in = 用于：{ $parts }
about-tp-artifact-server = 服务器
about-tp-artifact-web = Web 应用
about-tp-artifact-plugins = 插件
about-tp-artifact-desktop = 桌面应用
about-tp-artifact-mobile = 移动应用
about-tp-note-bundled = 与 Web 应用打包在一起。
about-tp-note-fonts-pull = 未打包：由服务器运维人员通过 `mailwoman fonts pull` 下载。
about-tp-note-first-party = 为 Mailwoman 制作，采用本项目自己的许可证。
about-tp-note-static-libc = 链接到 FROM-scratch 镜像的服务器二进制文件中。
about-tp-note-alpine-ca = FROM-scratch 镜像的 CA 证书包，取自 Alpine Linux。
about-tp-in-images = 位于：{ $images }
about-tp-image-runtime = distroless 运行时镜像
about-tp-image-static = FROM-scratch 镜像

about-tp-repository = 源代码
about-tp-homepage = 主页
about-tp-new-tab = { $name } 的链接（在新标签页中打开）

about-tp-text-toggle =
    { $count ->
       *[other] 许可证文本（{ $count }）
    }
about-tp-text-open-for = { $name } 的许可证文本
about-tp-dialog-title = 许可证文本：{ $name } { $version }
about-tp-close = 关闭
about-tp-text-loading = 正在加载许可证文本…
about-tp-text-error = 无法加载许可证文本。
about-tp-retry = 重试
about-tp-text-missing = 列表中缺少此文本。请使用 scripts/about/gen-third-party.mjs 重新生成。
about-tp-text-label = { $name } 的许可证文本：{ $file }
about-tp-standard = { $licence } 许可证
about-tp-standard-note = （标准文本，软件包中无此文件）
about-tp-no-text = 此条目没有可用的许可证文本。
