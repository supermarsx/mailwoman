# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-features. Source: ../en/admin-features.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-features-title = 機能プロファイル
admin-features-intro = プロファイルは、名前を付けた機能スイッチのセットです。ドメイン、グループ、ユーザーに割り当てられます。個別の割り当ては、ネイティブのメールボックスと、検証済みのプロキシまたは SSO の ID に対応しており、Mailwoman の機能へのアクセスを制御します。
admin-features-new = 新しいプロファイル
admin-features-search = プロファイルを検索
admin-features-empty = 機能プロファイルはまだありません。
admin-features-no-match = 検索に一致するプロファイルはありません。
admin-features-load-error = 機能カタログを読み込めませんでした。
admin-features-not-loaded = 機能カタログはまだ読み込まれていません。ページを再読み込みして、もう一度お試しください。
admin-features-stale = 別の管理者が機能カタログを変更しました。再読み込みしたので、値を確認してからもう一度保存してください。
admin-features-not-found = id が { $id } のプロファイルはありません。
admin-features-col-name = 名前
admin-features-col-values = スイッチ
admin-features-col-assigned = 割り当て先
admin-features-values-count = { $count ->
   *[other] { $count } 件のスイッチを設定済み
}
admin-features-assigned-count = { $count ->
    [0] 未割り当て
   *[other] { $count } か所
}
admin-features-editor = プロファイルの設定
admin-features-name = プロファイル名
admin-features-name-help = 管理者にのみ表示されます。128 文字までです。
admin-features-name-invalid = 1 ～ 128 文字の名前を入力してください。
admin-features-switches = 機能スイッチ
admin-features-switches-help = 「未設定」にすると、その機能は次のプロファイル、ドメイン、またはサーバーのデフォルトに委ねられます。ロックされた値は、ユーザーや下位のレベルでは変更できません。
admin-features-col-feature = 機能
admin-features-col-value = 値
admin-features-col-lock = ロック
admin-features-value-for = { $feature } の値
admin-features-unset = 未設定
admin-features-on = 許可
admin-features-off = 不許可
admin-features-lock = ロック済み
admin-features-save = プロファイルを保存
admin-features-create = プロファイルを作成
admin-features-delete = プロファイルを削除
admin-features-delete-title = プロファイル { $name } を削除しますか？
admin-features-delete-body = { $count ->
    [0] このプロファイルはどこにも割り当てられていません。スイッチはすぐに適用されなくなります。
   *[other] このプロファイルは割り当て先の { $count } か所から削除され、スイッチはすぐに適用されなくなります。
}
admin-features-saved = プロファイル { $name } を保存しました。
admin-features-deleted = プロファイル { $name } を削除しました。
admin-features-assigned = プロファイル { $name } を割り当てました。
admin-features-unassigned = { $name } の割り当てを解除しました。
admin-features-assignments = 割り当て先
admin-features-assignments-help = プロファイルは優先度の高い順に適用されます。優先度が同じ場合は、ユーザーへの割り当てがグループより、グループがドメインより先に適用されます。
admin-features-assignments-empty = このプロファイルはまだ割り当てられていません。
admin-features-assign-form = このプロファイルを割り当てる
admin-features-col-scope = 適用範囲
admin-features-col-target = 名前
admin-features-col-priority = 優先度
admin-features-col-actions = 操作
admin-features-scope-domain = ドメイン
admin-features-scope-group = グループ
admin-features-scope-user = ユーザー
admin-features-assign = 割り当て
admin-features-unassign = 削除
admin-features-no-domain = ドメインが設定されていません。
admin-features-no-group = ユーザーグループがありません。
admin-features-no-user = サインインしたメールボックスアカウントはまだありません。
admin-features-tools = プロファイルツール
admin-features-preset = プリセットから開始
admin-features-preset-choose = プリセットを選択
admin-features-preset-apply = プリセットを適用
admin-features-preset-applied = プリセット { $name } を適用しました。値を確認して保存してください。
admin-features-preset-full = フル
admin-features-preset-standard = 標準
admin-features-preset-mail-only = メールのみ
admin-features-preset-minimal = 最小 / キオスク
admin-features-import = JSON をインポート
admin-features-export = JSON をエクスポート
admin-features-exported = プロファイルを JSON ファイルとしてエクスポートしました。
admin-features-imported = { $count ->
   *[other] { $count } 件の値をインポートしました。確認して保存してください。
}
admin-features-import-json = このファイルは有効な JSON ではありません。
admin-features-import-format = このファイルは Mailwoman の機能プロファイルではありません。
admin-features-import-version = この機能プロファイルのファイルは新しいバージョンのもので、インポートできません。
admin-features-clone = 複製
admin-features-clone-name = { $name } のコピー
admin-features-filter = 機能を探す
admin-features-filter-placeholder = 名前、設定、またはキー
admin-features-filter-none = 検索に一致する機能はありません。
admin-features-diff-summary = { $differs ->
   *[other] { $differs } 件の値がサーバーのデフォルトと異なります
} · { $same } 件は同じ値です
admin-features-differs = デフォルトと異なる
admin-features-default-is = デフォルト：{ $value }
admin-features-fix-values = 保存する前に、無効と表示された値を修正してください。
admin-features-allow-all = すべて許可
admin-features-deny-all = すべて不許可
admin-features-clear-all = クリア
admin-features-group-allow = { $group } のすべての機能を許可
admin-features-group-deny = { $group } のすべての機能を不許可
admin-features-group-clear = { $group } のすべての値をクリア
admin-features-enforcement-ui = アプリ内でのみ非表示
admin-features-enforcement-partial = 一部の制限はまだカウントされません
admin-features-other = プロファイルに設定できるその他の項目
admin-features-other-help = 管理者がプロファイルごとに設定できるその他すべての項目です。フィールドを空にすると、サーバーの値のままになります。
admin-features-cat-compose = メールと作成
admin-features-cat-reading = 閲覧とプライバシー
admin-features-cat-views = 表示と検索
admin-features-cat-organise = 整理
admin-features-cat-pim = カレンダー、連絡先、タスク、ノート、キー
admin-features-cat-data = データ
admin-features-cat-accounts = アカウントとアクセス
admin-features-cat-integrations = 連携
admin-features-cat-developer = 開発者アクセス
admin-features-cat-personal = パーソナライズ
admin-features-feature-compose-sending = 送信の制限
admin-features-feature-compose-sending-help = 1 通あたりの受信者数が上限を超えると、サーバーが拒否します。1 時間あたりと 1 日あたりのメッセージ数はプロファイルに保存されますが、まだカウントされません。
admin-features-feature-storage = ストレージ容量
admin-features-feature-storage-help = ストレージの上限はプロファイルに保存されます。メールサーバー自体の容量制限も引き続き適用されます。
admin-features-feature-access-protocols = メールアクセスのプロトコル
admin-features-feature-access-protocols-help = これらのユーザーがサインインに使えるメールサーバーのプロトコルを、カンマ区切りで指定します：imap, pop3, ews。
admin-features-feature-accounts-linked = リンクされたメールボックス
admin-features-feature-calendar = カレンダー
admin-features-feature-contacts = 連絡先
admin-features-feature-tasks = タスク
admin-features-feature-notes = ノート
admin-features-preview = 有効な値
admin-features-preview-help = すべてのプロファイル、グループ、ドメインの値、ロックを適用した後に、各機能が 1 人のユーザー、または 1 つのドメインのユーザーに対してどう解決されるかを表示します。
admin-features-preview-kind = プレビュー対象
admin-features-preview-target = 対象
admin-features-preview-show = 表示
admin-features-preview-loading = 読み込み中…
admin-features-preview-state = 有効
admin-features-preview-from = 決定元
admin-features-preview-profiles = 適用されるプロファイル（先に一致したものが優先）：{ $names }
admin-features-preview-no-profiles = 適用されるプロファイルはありません。ドメインとサーバーの値で決まります。
admin-features-source-default = 組み込みのデフォルト
admin-features-source-env-fallback = 環境のデフォルト
admin-features-source-env-pin = 環境（固定）
admin-features-source-global = サーバーの値
admin-features-source-domain = ドメインの値
admin-features-source-profile = プロファイル
admin-features-source-profile-named = プロファイル { $name }
admin-features-source-group = グループの値
admin-features-source-admin-user = このユーザーに設定された値
admin-features-source-user = ユーザー自身の選択
admin-features-source-locked = { $source }（ロック済み）
admin-retention-title = 監査ログの保持
admin-retention-intro = カテゴリごとに、監査エントリーを保持する期間を設定します。独自の値がないカテゴリはデフォルトに従い、デフォルトもない場合はすべてのエントリーを保持します。法的保全を設定したカテゴリは削除されず、デフォルトに法的保全を設定すると、すべてのカテゴリで削除されません。変更と削除は監査ログに記録されます。
admin-retention-loading = 保持ポリシーを読み込み中…
admin-retention-unavailable = このサーバーでは監査の保持を利用できません。
admin-retention-col-category = カテゴリ
admin-retention-col-kept = 保持
admin-retention-col-change = 変更
admin-retention-cat-default = デフォルト
admin-retention-cat-admin = 管理者の操作
admin-retention-cat-security = セキュリティイベント
admin-retention-cat-alert = ユーザーへのアラート
admin-retention-forever = すべてのエントリー
admin-retention-days = { $days ->
   *[other] { $days } 日
}
admin-retention-follows-default = デフォルト：{ $kept }
admin-retention-hold-on = 法的保全
admin-retention-read-only = 保持設定を変更するには settings.write 権限が必要です。
admin-retention-edit-for = { $category } の保持
admin-retention-keep = 保持
admin-retention-mode-inherit = デフォルトに従う
admin-retention-mode-forever = すべてのエントリー
admin-retention-mode-days = 日数を指定
admin-retention-days-label = 日数
admin-retention-hold = 法的保全
admin-retention-save = 保存
admin-retention-bounds = { $min } ～ { $max } の整数を入力してください。
admin-retention-saved = { $category } の保持設定を保存しました。
admin-retention-task-none = 削除タスクはまだありません。日数を初めて保存したときに作成され、毎日 03:15 UTC に実行されます。
admin-retention-task-disabled = スケジュールされた削除タスクは無効です。エントリーは「今すぐ削除」を使ったときだけ削除されます。
admin-retention-task-next = 次回の予定削除：{ $when }。
admin-retention-task-daily = 毎日 03:15 UTC に削除されます。
admin-retention-last = エントリーを削除した直近の実行：{ $when }。
admin-retention-never = まだ削除されたエントリーはありません。
admin-retention-prune = 今すぐ削除
admin-retention-prune-title = 監査ログを今すぐ削除しますか？
admin-retention-prune-body = カテゴリの保持期間より古いエントリーが削除されます。すべてのエントリーを保持するカテゴリや、法的保全の対象のカテゴリは変更されません。この操作は元に戻せません。
admin-retention-pruned = { $count ->
    [0] 削除できるほど古いエントリーはありませんでした。
   *[other] { $count } 件のエントリーを削除しました。
}
admin-auth-attempted = { $name } として試行
admin-features-feature-compose-open-tracking = メッセージの開封追跡
admin-features-feature-compose-open-tracking-help = 対象となる書式付きメッセージの開封を、サーバーのトラッキング画像で記録することを許可します。公開 URL の設定が必要です。
admin-features-feature-reading-original-html = 元の HTML 表示
admin-features-feature-reading-original-html-help = ユーザーが、隔離されたリーダーフレーム内で、個別のメッセージについて元の HTML を使うよう選べるようにします。デフォルトは引き続きサニタイズ済み HTML です。
admin-features-inherit = 継承
admin-features-override = 値を設定
admin-features-setting-mode = { $setting } の値のモード
admin-features-inherit-help = 継承すると、このプロファイルの値とロックが削除されます。次に該当するプロファイル、ドメイン、またはサーバーの値で決まり、有効な値のプレビューに結果とそのソースが表示されます。
admin-features-domain-alias = { $alias }（{ $domain } のエイリアス）
admin-features-domain-aliases = { $domain }（エイリアス：{ $aliases }）
