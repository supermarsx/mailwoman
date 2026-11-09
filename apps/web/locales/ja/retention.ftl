# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: retention. Source: ../en/retention.ftl


# -- Machine translation (2026-11): added to match en ---------------------
retention-title = 保持ルール
retention-intro = 一定の日数が経過した自分のメールを移動または削除するルールです。ルールでは、フォルダー、ラベル、差出人、サイズ、既読かどうか、添付ファイルの有無を条件にできます。フラグ付きのメールと「keep」ラベルのメールは、設定を変えない限り対象外です。削除したメールはまずゴミ箱に移動し、元に戻せる期間が過ぎると完全に削除されます。
retention-disabled = このサーバーでは保持ルールがオフになっています。既存のルールは実行されません。
retention-never = なし
retention-error = 保持設定を読み込めないか、保存できませんでした。もう一度お試しください。
retention-last-run = 前回の実行: { $when }
retention-next-run = 次回の実行は約 { $days } 日後
retention-next-run-unknown = 次回の実行はまだ予定されていません
retention-enabled = 自分の保持ルールを実行する
retention-trash-first = 削除したメールをまずゴミ箱に移動する
retention-trash-forced = このサーバーでは、メールは必ずまずゴミ箱に移動され、元に戻せる期間が過ぎると完全に削除されます。
retention-undo = 元に戻せる期間（日）
retention-undo-help = ルールでゴミ箱に移動されたメールは、この期間だけ残ってから完全に削除されます。0～{ $max } 日。
retention-pause = すべてのルールを30日間一時停止
retention-paused-until = { $when } まで一時停止中
retention-resume = 再開
retention-policy = このサーバーで許可されていること
retention-limits-both = メールを削除するルールは、最短 { $min } 日、最長 { $max } 日待ってから実行されます。
retention-limits-min = メールを削除するルールは、最短 { $min } 日待ってから実行されます。
retention-max-rules = ルールは最大 { $count } 件まで保存できます。
retention-permanent-allowed = ゴミ箱の手順をオフにすると、ルールでメールを完全に削除できます。
retention-permanent-denied = ルールでメールを完全に削除することはできません。メールはゴミ箱に移動し、{ $days } 日後に完全に削除されます。
retention-protected = ルールが対象にしないフォルダー: { $folders }
retention-mandatory = 管理者による設定: { $folder } 内の { $days } 日より古いメールは削除されます。
retention-rules = 自分のルール
retention-add = 新しいルール
retention-no-rules = ルールはまだありません。下のテンプレートを使うか、ルールを追加してください。
retention-rule-paused = 一時停止中
retention-rule-on = オン
retention-rule-enabled = ルール { $name } はオンです
retention-edit-short = 編集
retention-delete = 削除
retention-new = 新しいルール
retention-edit = ルールを編集
retention-name = 名前
retention-action = 実行する処理
retention-action-keep = ずっと保持
retention-action-delete = 削除
retention-action-trash = ゴミ箱に移動
retention-action-archive = アーカイブに移動
retention-action-move = フォルダーに移動
retention-action-markRead = 既読にする
retention-days = 経過日数
retention-target = フォルダー
retention-choose-folder = フォルダーを選択
retention-scope = 対象のメール
retention-folder = フォルダー
retention-any-folder = すべてのフォルダー
retention-subfolders = サブフォルダーを含める
retention-labels = ラベル（カンマ区切り）
retention-senders = 差出人アドレス（カンマ区切り）
retention-domains = 差出人ドメイン（カンマ区切り）
retention-min-size = 最小サイズ（KB）
retention-max-size = 最大サイズ（KB）
retention-read = 既読状態
retention-read-any = 既読・未読どちらも
retention-read-read = 既読のみ
retention-read-unread = 未読のみ
retention-attachment = 添付ファイル
retention-attachment-any = 有無を問わない
retention-attachment-with = 添付ファイルあり
retention-attachment-without = 添付ファイルなし
retention-exclude = 対象外にする
retention-exclude-flagged = フラグ付き（スター付き）のメール
retention-exclude-unread = 未読のメール
retention-exclude-tags = これらのラベルが付いたメール（カンマ区切り）
retention-exclude-note = 法的保全の対象となっているメールは、ここでの設定にかかわらず変更されません。
retention-role-inbox = 受信トレイ
retention-role-sent = 送信済み
retention-role-archive = アーカイブ
retention-role-junk = スパム
retention-role-trash = ゴミ箱
retention-role-drafts = 下書き
retention-btn-preview = プレビュー
retention-save = 保存
retention-cancel = キャンセル
retention-preview-none = 変更されるメッセージはありません。
retention-preview-count = { $count } 件のメッセージが変更されます。
retention-preview-truncated = このプレビューでは、メールボックスの一部のみを読み取りました。
retention-preview-sample = 例
retention-no-subject = （件名なし）
retention-do-trash = ゴミ箱へ
retention-do-move = 移動
retention-do-delete = 削除
retention-do-markRead = 既読
retention-skipped-legalHold = アカウントは法的保全の対象です。何も変更されません。
retention-skipped-restricted = アカウントの処理が制限されています。何も変更されません。
retention-skipped-noRules = 有効なルールがなく、管理者のルールも設定されていません。
retention-error-name = ルールに名前を付けてください。
retention-error-days = 日数は0～3650の整数で指定してください。
retention-error-min = メールを削除するルールは、最低 { $days } 日待つ必要があります。
retention-error-max = メールを削除するルールの待機期間は、最長 { $days } 日です。
retention-error-target = メールの移動先フォルダーを選択してください。
retention-summary-keep = ずっと保持
retention-summary-delete = { $days } 日後にゴミ箱へ移動し、その後完全に削除
retention-summary-delete-permanent = { $days } 日後に完全に削除
retention-summary-trash = { $days } 日後にゴミ箱へ移動
retention-summary-archive = { $days } 日後にアーカイブへ移動
retention-summary-move = { $days } 日後にフォルダーへ移動
retention-summary-markRead = { $days } 日後に既読にする
retention-templates = テンプレート
retention-templates-help = テンプレートを使うと、あらかじめ用意されたルールを追加できます。利用前にラベルと日数を確認してください。
retention-adopt = このルールを追加
retention-run-heading = 今すぐ実行
retention-preview-all = すべてのルールをプレビュー
retention-btn-run = 自分のルールを今すぐ実行
retention-run-result = { $count } 件のメッセージを変更しました。
retention-history = 最近の実行
retention-history-when = 日時
retention-history-matched = 該当
retention-history-changed = 変更
retention-history-errors = エラー
retention-context-action = 保持ルール…

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
pim-cal-calendars = カレンダー

pim-cal-days = イベント終了から数日後

pim-cal-enabled = 終了したイベントを削除する

pim-cal-help = 選択したカレンダー内のイベントは、終了後設定日数後に削除されます。定期的なイベントは削除されません。

pim-cal-title = カレンダーイベント

pim-days-range = { $min } 日から { $max } 日の間で選択します。

pim-held-title = 削除されたメモは復元可能です

pim-held-until = { $when }まで保持

pim-limits = { $min } 日から { $max } 日の間。

pim-notes-days = メモが最後に変更されてからの日数

pim-notes-enabled = 触れられていないメモを削除する

pim-notes-help = 一定の日数変更しなかったメモを削除します。ピン留めされたメモは保持されます。削除したメモは、{ $days } 日間復元できます。

pim-notes-title = 注意事項

pim-off = 管理者によってオフにされています。

pim-preview = プレビュー

pim-restore = 復元

pim-result = { $events } イベントと { $notes } メモ。

pim-run = 今すぐ実行

pim-run-title = 今すぐ実行

pim-save = 保存

pim-skipped = 管理者がオフにしたため、{ $count } 個の選択肢がスキップされました。

pim-untitled = 無題のメモ

retention-action-stripAttachments = 大きな添付ファイルを削除する

retention-do-stripAttachments = 添付ファイルが削除されました

retention-error-strip-disabled = 添付ファイルの削除は管理者によって無効にされています。

retention-error-strip-min = 添付ファイルを削除するルールは、少なくとも { $days } 日待つ必要があります。

retention-fd-action = { $folder } のメールはどうなりますか

retention-fd-days = { $folder } が処理されるまでの日数

retention-fd-help = メールが移動または削除されるまでに、フォルダー内にメールが保持される期間を設定します。 「継承」に設定されたフォルダーには、独自のデフォルトはありません。上記のルールと管理者の制限は引き続き適用されます。サブフォルダーはそのフォルダーの後に続きます。

retention-fd-inherit = 継承する

retention-fd-mandatory = 管理者はこのフォルダーを { $days } 日間保持します

retention-fd-protected = 管理者によって保護されています

retention-fd-save = フォルダーのデフォルトを保存する

retention-fd-title = フォルダーのデフォルト

retention-rule-strip-off = 管理者によってオフにされました

retention-strip-allowed = ルールにより、少なくとも { $days } 日経過したメールから少なくとも { $kb } KB の添付ファイルが削除される場合があります。各添付ファイルは短いメモに置き換えられます。

retention-strip-denied = 添付ファイルの削除は管理者によってオフにされています。

retention-strip-done = { $count } メッセージから添付ファイルが削除され、{ $size } が解放されました。

retention-strip-inline-kept = メッセージ内に表示される画像は保持されます。

retention-strip-inline-removed = メッセージ内に表示されていた画像も削除されます。

retention-strip-limits = 少なくとも { $days } 日前のメールから、少なくとも { $kb } KB の添付ファイルを削除します。オリジナルは { $undo } 日間ゴミ箱に残ります。

retention-strip-preview = { $count } メッセージの添付ファイルが失われ、{ $size } が解放されます。

retention-strip-preview-estimate = 約 { $count } 個のメッセージの添付ファイルが失われ、最大 { $size } 個のメッセージが解放されます。

retention-strip-skipped = 管理者がこれをオフにしたため、添付ファイルを削除する { $count } ルールがスキップされました。

retention-summary-stripAttachments = { $days } 日後、大きな添付ファイルが削除されました
