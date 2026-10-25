# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: rules. Source: ../en/rules.ftl

rules-title = ルールとフィルター
rules-intro = メールのフィルタースクリプトとルールを管理します。利用できる機能はアカウントと管理者の設定によって異なります。
rules-empty = ルールはまだありません。
rules-unavailable = このメールサーバーはメールルールに対応していません。
rules-load-failed = ルールを読み込めませんでした。
rules-new = 新しいルール
rules-list-label = メールルール
rules-enabled = 有効
rules-delete-named = ルール { $name } を削除

rules-builder-title = ルールビルダー
rules-builder-hint = 編集するルールを選択するか、新しく作成してください。
rules-name-label = ルール名
rules-match-legend = 実行条件
rules-match-all = すべての条件に一致
rules-match-any = いずれかの条件に一致
rules-conditions-legend = 条件
rules-actions-legend = アクション
rules-add-condition = 条件を追加
rules-add-action = アクションを追加
rules-remove-condition = 条件を削除
rules-remove-action = アクションを削除
rules-cond-type-label = フィールド
rules-cond-op-label = 演算子
rules-cond-value-label = 一致する値
rules-act-type-label = アクション
rules-act-value-label = アクションの値
rules-save = ルールを保存
rules-editing = { $name } を編集中
rules-save-refused = ルールは保存されませんでした: { $reason }

rules-cond-from = 差出人
rules-cond-to = 宛先
rules-cond-subject = 件名
rules-cond-thread-unsupported = スレッド { $op } { $value }: スレッドの条件には対応していないため、この条件を削除するまでルールを保存できません。
rules-op-contains = を含む
rules-op-is = と一致

rules-act-move = メールボックスに移動
rules-act-tag = タグを追加
rules-act-archive = アーカイブ
rules-act-suppress = 通知しない
rules-act-stop = 処理を停止

rules-runs-server = サーバー (Sieve)
rules-runs-server-detail = メールサーバーにアップロードされ、そこで適用されます。
rules-runs-engine = エンジン
rules-runs-engine-detail = 新しい受信メッセージの同期時に適用されます。返信と移動は次の同期まで待機します。

rules-panes-label = ルールツール
rules-tab-builder = ビルダー
rules-tab-raw = Sieve ソース
rules-tab-dryrun = テスト実行

rules-raw-label = Sieve のソース
rules-raw-reset = 生成された内容に戻す
rules-lint-clean = 問題は見つかりませんでした。

rules-dryrun-title = テスト実行
rules-dryrun-help = サンプルのメッセージを入力すると、どのルールが適用されるかを確認できます。
rules-dryrun-results = テスト実行の結果
rules-dryrun-none = 評価する有効なルールはありません。
rules-dryrun-nomatch = 一致なし
rules-dryrun-stopped = 未到達 (前のルールで処理が停止しました)

rules-scripts-label = Sieve スクリプト
rules-script-active = 有効
rules-script-new = 新しいスクリプト
rules-script-export = スクリプトをエクスポート
rules-script-import = スクリプトをインポート
rules-script-too-large = スクリプトがサイズ制限を超えています。
rules-script-read-only = 管理者によりスクリプトの直接編集が無効になっています。
rules-script-validate = スクリプトを検証
rules-script-deactivate = 無効にする
rules-script-activate = 有効にする
rules-script-saved = スクリプトを保存しました。

rules-vacation-title = 不在時の返信
rules-vacation-enabled = 自動返信を有効にする
rules-vacation-from = 開始（UTC）
rules-vacation-to = 終了（UTC）
rules-vacation-body = 返信の本文
rules-vacation-saved = 不在時の返信を保存しました。
rules-script-template = スクリプトのテンプレート
rules-forward-title = 転送
rules-forward-address = 転送先
rules-forward-copy = このアカウントにコピーを残す
rules-forward-refused = 管理者はこのドメインへの転送を許可していません。
