# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: about. Source: ../en/about.ftl

about-tp-label = サードパーティのライブラリとライセンス
about-tp-search = 名前、バージョン、ライセンスで検索
about-tp-filter = ライセンスで絞り込み
about-tp-filter-all = すべて ({ $n })
about-tp-filter-licence = { $licence } ({ $n })
about-tp-showing =
    { $count ->
        [0] 一致する項目はありません。
       *[other] { $total }件中{ $shown }件を表示しています。
    }
about-tp-empty = この検索とライセンスに一致する項目はありません。
about-tp-clear = 検索と絞り込みをクリア
about-tp-more = さらに表示 (残り{ $n }件)

about-tp-kind-crate = Rust クレート
about-tp-kind-npm = npm パッケージ
about-tp-kind-font = フォント
about-tp-kind-icons = アイコン
about-tp-kind-image = 画像
about-tp-kind-os = イメージパッケージ

about-tp-group-count = { $title } ({ $n })
about-tp-group-rust = サーバー、Web アプリ、プラグインの Rust クレート
about-tp-group-shells = デスクトップアプリとモバイルアプリのみの Rust クレート
about-tp-group-web = Web アプリの npm パッケージ
about-tp-group-assets = フォント、アイコン、画像
about-tp-group-image = サーバーイメージのファイル
about-tp-group-build = ビルドツール: ビルド時に実行され、配布物には含まれません

about-tp-licence = ライセンス:
about-tp-licence-none = 未宣言
about-tp-licence-in-file = 著作権ファイルを参照
about-tp-used-in = 使用箇所: { $parts }
about-tp-artifact-server = サーバー
about-tp-artifact-web = Web アプリ
about-tp-artifact-plugins = プラグイン
about-tp-artifact-desktop = デスクトップアプリ
about-tp-artifact-mobile = モバイルアプリ
about-tp-note-bundled = Web アプリに同梱されています。
about-tp-note-fonts-pull = 同梱されていません。サーバー管理者が `mailwoman fonts pull` でダウンロードします。
about-tp-note-first-party = Mailwoman 向けに作成され、プロジェクト独自のライセンスで提供されています。
about-tp-note-static-libc = FROM-scratch イメージのサーバーバイナリにリンクされています。
about-tp-note-alpine-ca = FROM-scratch イメージの CA 証明書バンドルです。Alpine Linux から取得しています。
about-tp-in-images = 含まれるイメージ: { $images }
about-tp-image-runtime = distroless ランタイムイメージ
about-tp-image-static = FROM-scratch イメージ

about-tp-repository = ソースコード
about-tp-homepage = ホームページ
about-tp-new-tab = { $name } (新しいタブで開きます)

about-tp-text-toggle =
    { $count ->
       *[other] ライセンス文 ({ $count })
    }
about-tp-text-open-for = { $name } のライセンス文
about-tp-dialog-title = ライセンス文: { $name } { $version }
about-tp-close = 閉じる
about-tp-text-loading = ライセンス文を読み込んでいます…
about-tp-text-error = ライセンス文を読み込めませんでした。
about-tp-retry = 再試行
about-tp-text-missing = この文はリストにありません。scripts/about/gen-third-party.mjs で再生成してください。
about-tp-text-label = { $name } のライセンス文: { $file }
about-tp-standard = { $licence } ライセンス
about-tp-standard-note = (標準文。パッケージ内にファイルはありません)
about-tp-no-text = この項目のライセンス文はありません。
