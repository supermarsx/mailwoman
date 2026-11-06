# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-webhooks. Source: ../en/admin-webhooks.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-webhooks-title = Webhook
admin-webhooks-intro = 送信 webhook は、あるアカウントで何かが起きたときに、署名付きの通知を URL に送ります。受信 webhook は、別のシステムがあるアカウントでメールルールのアクションを実行できるようにします。
admin-webhooks-empty = webhook はまだありません。
admin-webhooks-unavailable = このサーバーは webhook の管理を提供していません。
admin-webhooks-load-error = webhook を読み込めませんでした。
admin-webhooks-direction-outbound = 送信
admin-webhooks-direction-inbound = 受信
admin-webhooks-col-direction = 方向
admin-webhooks-col-label = 名前
admin-webhooks-col-account = アカウント
admin-webhooks-col-target = 送信先または許可
admin-webhooks-col-created = 作成日
admin-webhooks-no-label = （名前なし）
admin-webhooks-created-by = 作成者：{ $who }
admin-webhooks-last-used = 最終使用：{ $when }
admin-webhooks-never-used = 未使用
admin-webhooks-delete = 削除
admin-webhooks-delete-named = webhook { $name } を削除
admin-webhooks-delete-title = webhook { $name } を削除しますか？
admin-webhooks-delete-body = この webhook への配信は停止し、そのシークレットで署名された呼び出しは拒否されます。この操作は元に戻せません。新しい webhook には新しいシークレットが付与されます。
admin-webhooks-deleted = webhook を削除しました。
admin-webhooks-add = webhook を追加
admin-webhooks-add-direction = 方向
admin-webhooks-add-outbound-help = Mailwoman があなたの URL を呼び出します。
admin-webhooks-add-inbound-help = あなたのシステムが Mailwoman を呼び出します。
admin-webhooks-add-account = アカウント
admin-webhooks-add-account-help = イベントを送信するアカウント、または受信呼び出しが操作できる唯一のアカウントです。
admin-webhooks-add-label = 名前
admin-webhooks-add-label-help = 他の人が判別できるよう、webhook の用途を記入します。任意です。
admin-webhooks-add-url = URL
admin-webhooks-add-url-help = 公開インターネットのアドレスに到達できる必要があります。プライベート、ループバック、リンクローカルのアドレスは拒否されます。
admin-webhooks-add-events = イベント
admin-webhooks-add-actions = 許可するアクション
admin-webhooks-add-submit = webhook を作成
admin-webhooks-add-saving = 作成中…
admin-webhooks-add-check = チェック
admin-webhooks-add-check-ok = URL はチェックに合格しました。何も保存されていません。
admin-webhooks-event-message-received = 新着メッセージ
admin-webhooks-event-message-flags = メッセージが既読、フラグ付き、またはラベル付けされた
admin-webhooks-event-message-moved = メッセージが移動された
admin-webhooks-event-message-deleted = メッセージが削除された
admin-webhooks-event-submission-sent = メッセージが送信された
admin-webhooks-event-submission-failed = メッセージを送信できなかった
admin-webhooks-event-pim-changed = カレンダー、連絡先、タスク、メモが変更された
admin-webhooks-action-run-rules = メッセージの記述に対して、アカウントのメールルールを実行
admin-webhooks-action-sieve = メッセージに1つのルールアクションを適用
admin-webhooks-secret-title = { $name } のシークレット
admin-webhooks-secret-once = このシークレットを今すぐコピーしてください。再表示されません。
admin-webhooks-secret-outbound = 各配信には、X-Mailwoman-Signature: sha256= に続けて、このシークレットによるリクエスト本文の HMAC-SHA256 が付きます。
admin-webhooks-secret-inbound = X-Mailwoman-Webhook: { $id }、X-Mailwoman-Timestamp（Unix 秒）、X-Mailwoman-Nonce（英字・数字・- または _ の 16〜128 文字、再利用不可）、および X-Mailwoman-Signature: sha256= に続けて、このシークレットによる timestamp.nonce.body の HMAC-SHA256 を送信してください。5分以上ずれた呼び出しや、nonce を再利用した呼び出しは拒否されます。
admin-webhooks-secret-done = コピーしました
admin-webhooks-error-stale = このページを開いた後に、誰かが webhook を変更しました。一覧を再読み込みしたので、もう一度お試しください。
admin-webhooks-error-account = アカウントを入力してください。
admin-webhooks-error-url = https:// または http:// で始まる URL を入力してください。
admin-webhooks-error-events = イベントを1つ以上選択してください。
admin-webhooks-error-actions = アクションを1つ以上選択してください。
