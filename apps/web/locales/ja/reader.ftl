# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: reader. Source: ../en/reader.ftl


# -- Machine translation (2026-11): added to match en ---------------------
reader-settings-title = メッセージの表示
reader-settings-help = メッセージの表示方法を選択します。上書きを選ぶまでは、管理者の既定値が適用されます。メッセージの内容は常にアプリケーションから分離されます。
reader-colorMode = メッセージの色
reader-colorMode-help = 「自動」はダークテーマでメッセージの色を調整します。「オリジナル」は送信者の色を保持し、「テーマに合わせる」は現在のテーマを使用し、「反転」は画像の色を保ったまま色を反転します。
reader-fontSize = 閲覧時のフォントサイズ（px）
reader-fontSize-help = メッセージ本文の基本フォントサイズで、12 から 24 ピクセルです。見出しのサイズは送信者の書式で定義される場合があります。
reader-lineHeight = 閲覧時の行の高さ
reader-lineHeight-help = メッセージ本文の行間で、フォントサイズの 1.2 倍から 2.2 倍です。
reader-contentWidth = 閲覧時の幅
reader-contentWidth-help = 長い行を制限して読みやすくするか、メッセージペイン全体を使用します。
reader-loading = 読み込み中の表示
reader-loading-help = 落ち着いたプレースホルダー行、テキストインジケーター、または視覚的な読み込み表示なしから選択します。スクリーンリーダーには引き続き読み込み状態が通知されます。
reader-loadingDelayMs = 読み込みインジケーターの遅延（ミリ秒）
reader-loadingDelayMs-help = 短時間で終わるリクエストでインジケーターが一瞬だけ表示されるのを防ぎます。遅延するのはインジケーターだけで、メッセージ自体は遅延しません。
reader-option-auto = テーマに合わせて自動
reader-option-original = メッセージの元の色
reader-option-adaptive = テーマに合わせる
reader-option-invert = メッセージの色を反転
reader-option-comfortable = 標準
reader-option-wide = 広め
reader-option-full = ペイン全体
reader-option-skeleton = プレースホルダー行
reader-option-indicator = テキストインジケーター
reader-option-none = 視覚的な表示なし
reader-inherit = 管理者の既定値を使用
reader-actionStyle = メッセージ操作のリボン
reader-actionStyle-help = コンパクトなアイコンにすると、メッセージ用の領域が広がります。どの操作もキーボード用の名前と説明のツールチップを保持します。アイコンの横にテキストを表示したい場合は、ラベルを表示する設定を選んでください。
reader-option-icons = コンパクトなアイコン
reader-option-labels = アイコンとラベル
reader-zoom-group = メッセージのズーム
reader-zoom-in = 拡大
reader-zoom-in-help = メッセージを大きくします。サイズが変わるのはメッセージだけです。
reader-zoom-out = 縮小
reader-zoom-out-help = メッセージを小さくします。サイズが変わるのはメッセージだけです。
reader-zoom-level = { $level }%
reader-zoom-reset-label = メッセージのズーム { $level }%。100% にリセット
reader-zoom-reset-help = メッセージを 100% にリセット
reader-zoom-reset-menu = ズームをリセット（{ $level }%）
reader-zoom = メッセージのズーム（%）
reader-zoom-help = メッセージを開くときのサイズで、50% から 200% です。拡大縮小されるのはメッセージ本文のみで、アプリ自体は変わりません。印刷は常に 100% で行われます。メッセージ操作の横にあるズームボタンでも変更できます。
reader-attachments-summary = { $count ->
   *[other] { $count } 件の添付ファイル
} · { $size }
reader-attachments-show = 添付ファイルを表示
reader-attachments-hide = 添付ファイルを非表示
reader-attachmentsDensity = 添付ファイルの密度
reader-attachmentsDensity-help = 「コンパクト」は小さなファイルボタンを表示し、「標準」は展開時により大きなプレビューを表示します。
reader-option-compact = コンパクト
reader-attachmentsCollapsed = 添付ファイルを既定で折りたたむ
reader-attachmentsCollapsed-help = 各メッセージをコンパクトな添付ファイルの概要から表示します。この既定値を変えずに、特定のメッセージだけ展開できます。
reader-card-text = 文字と色
reader-card-text-help = メッセージ本文のサイズ、間隔、幅、色。
reader-card-actions = 操作と添付ファイル
reader-card-actions-help = メッセージ操作のリボンと、添付ファイルの初期状態。
reader-card-loading = 読み込み
reader-card-loading-help = メッセージを開いている間に表示される内容。
settings-mail-card-list = メッセージ一覧
settings-mail-card-list-help = 行の操作と、メッセージを操作したときの一覧の動作。

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
reader-attachments-download = { $name }をダウンロード

reader-attachments-download-all = すべてダウンロード

reader-attachments-download-all-blocked = 添付ファイル ポリシーにより、ここですべてのファイルがブロックされます

reader-attachments-download-all-busy = ZIP ファイルの準備

reader-attachments-download-all-label = すべての添付ファイルを ZIP ファイルとしてダウンロードします

reader-attachments-download-all-none = ダウンロードする添付ファイルはありません

reader-attachments-download-all-warned = { $count ->
       *[other] 確認が必要な { $count } ファイルが省略されました。一つずつダウンロードしてください。
    }

reader-attachments-download-blocked = { $name } はダウンロードできません: { $reason }

reader-attachments-download-label = { $name } ({ $size }) をダウンロード

reader-attachments-download-off = ダウンロードがオフになっています

reader-attachments-inline-group = { $count ->
       *[other] { $count } つのインライン画像
    }

reader-attachments-inline-summary = { $count ->
       *[other] { $count } つのインライン画像
    } · { $size }

reader-attachmentsBarClickToToggle = バーをクリックして展開または折りたたむ

reader-attachmentsBarClickToToggle-help = 添付ファイル バーの任意の場所をクリックすると、添付ファイル バーが切り替わります。オフの場合、矢印ボタンのみが機能します。

reader-attachmentsBarDownloadAll = 「すべてダウンロード」ボタン

reader-attachmentsBarDownloadAll-help = 添付ファイルを 1 つの ZIP ファイルとしてダウンロードするボタンをバーに表示します。

reader-attachmentsBarShowDownloadButtons = 添付ファイルのダウンロード ボタン

reader-attachmentsBarShowDownloadButtons-help = ホバーまたはフォーカス時に各添付ファイルのダウンロード ボタンを常に表示するか、表示しないようにします。

reader-attachmentsBarZipInlineImages = ZIP にインライン画像を含めます

reader-attachmentsBarZipInlineImages-help = これがオンになっていない限り、メッセージ テキスト内に表示される画像は [すべてダウンロード] から除外されます。

reader-card-attachments-bar = 添付ファイルバー

reader-card-attachments-bar-help = メッセージの添付ファイルの上にあるバーがどのように反応し、何を提供するのか。ロックされた行は、管理者によって設定されます。

reader-option-always = いつも

reader-option-hover = ホバーまたはフォーカス時

reader-option-never = 決してしない

reader-card-header = メッセージのヘッダー
reader-autoHideHeader = メッセージのヘッダーを自動的に隠す
reader-autoHideHeader-help = 各メッセージを開く際、差出人、宛先、メタデータの詳細を折りたたみます。件名と操作は表示したままです。ヘッダーを表示で詳細を確認できます。既定では無効です。
reader-header-show = ヘッダーを表示
reader-header-hide = ヘッダーを隠す
