# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: apikeys. Source: ../en/apikeys.ftl

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = API キー
apikeys-heading = API キー
apikeys-intro = スクリプトや連携用に、範囲を限定したキーを作成します。各キーはハッシュ化して保存され、一度だけ表示され、個別に失効できます。必要最小限の範囲を付与してください。

apikeys-label = ラベル
apikeys-label-aria = キーのラベル
apikeys-label-placeholder = 例：バックアップスクリプト

apikeys-create = キーを作成
apikeys-saved = 保存しました

apikeys-reveal-warning = このシークレットを今すぐコピーしてください — 表示されるのは一度だけで、後から取得することはできません。
apikeys-copy = コピー
apikeys-copy-aria = シークレットをクリップボードにコピー
apikeys-copied = クリップボードにコピーしました

apikeys-error-need-label = 後で識別できるよう、キーにラベルを付けてください
apikeys-error-create = キーを作成できませんでした
apikeys-error-revoke = キーを失効できませんでした

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = 既存のキー
apikeys-existing = 既存のキー
apikeys-none = キーはまだありません。
apikeys-created = 作成日 { $date }
apikeys-last-used = 最終使用 { $date }
apikeys-never-used = 未使用
apikeys-revoked-at = 失効日 { $date }
apikeys-revoke = 失効
apikeys-status-active = 有効
apikeys-status-revoked = 失効済み

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = 範囲
apikeys-capabilities = 権限
apikeys-cap-read = 読み取り
apikeys-cap-send = 送信
apikeys-cap-delete = 削除
apikeys-surface = 対象
apikeys-surface-mail-label = メール
apikeys-surface-pim-label = PIM（カレンダー / タスク / メモ / 連絡先）
apikeys-accounts = アカウント
apikeys-all-accounts = すべてのアカウント
apikeys-folders = フォルダー
apikeys-all-folders = すべてのフォルダー
apikeys-folder-ids = フォルダー ID（カンマ区切り）
apikeys-folder-subset-aria = 対象フォルダー
apikeys-constraints = 制約
apikeys-expiry = 有効期限（RFC 3339、空欄 = 無期限）
apikeys-expiry-aria = 有効期限
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = レート制限（リクエスト/分、空欄 = 無制限）
apikeys-rate-limit-aria = レート制限
apikeys-ip-allowlist = IP 許可リスト（CIDR/IP、カンマ区切り、空欄 = すべて）
apikeys-ip-allowlist-aria = IP 許可リスト
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = 読み取り
apikeys-verb-send = 送信
apikeys-verb-delete = 削除
apikeys-summary-mail = メール
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = 操作なし
apikeys-summary-nothing = なし
apikeys-summary-none = なし
apikeys-summary-verbs-on = { $surfaces } に対して { $verbs }
apikeys-summary-all-accounts = すべてのアカウント
apikeys-summary-accounts = アカウント：{ $ids }
apikeys-summary-all-folders = すべてのフォルダー
apikeys-summary-folders = フォルダー：{ $ids }
apikeys-summary-expires = 有効期限 { $date }
apikeys-summary-rate = { $n } リクエスト/分
apikeys-summary-ips = IP：{ $ips }
apikeys-summary-mcp-tools = MCP ツール：{ $tools }
apikeys-summary-unattended = 無人送信

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = MCP キー
apikeys-mcp-heading = MCP キー
apikeys-mcp-intro = AI エージェントに（Model Context Protocol 経由で）選択したツールだけを付与します。エージェントに返されるメール本文は、信頼できない入力として明示されます。ツールは 1 つずつ個別に付与されます。
apikeys-mcp-label-aria = MCP キーのラベル
apikeys-mcp-label-placeholder = 例：アシスタントエージェント
apikeys-mcp-create = MCP キーを作成
apikeys-mcp-reveal-warning = このシークレットを今すぐコピーしてください — 表示されるのは一度だけです。
apikeys-mcp-error-need-label = MCP キーにラベルを付けてください
apikeys-mcp-error-need-tool = ツールを 1 つ以上付与してください
apikeys-mcp-error-create = MCP キーを作成できませんでした

apikeys-tools = ツール
apikeys-tools-group = MCP ツール
apikeys-outbox-suffix = （送信トレイで確認）

apikeys-unattended-aria = 無人送信
apikeys-unattended-allow = 無人送信を許可（送信トレイを経由しない — 管理者の副署が必要）
apikeys-unattended-pending = 管理者がこのキーに副署するまで、無人送信は無効のままです。
apikeys-unattended-disclosure = 既定では、付与された送信は送信トレイに入り、アプリ内であなたが確認するまで待機します — 自動処理だけで送信することはできません。無人送信はこの人による確認の手順を「取り除き」、このキーで確認なしにメールを送信できるようにします。さらに、管理者によるキーへの副署が必要です。完全に信頼できる自動処理にのみ付与してください。無人送信キーが漏えいすると、確認なしにあなたとしてメールを送信される可能性があります。

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = メールを検索
apikeys-tool-mail-search-desc = メッセージを検索します。結果は信頼できない入力として明示されます。
apikeys-tool-mail-read-label = メールを読む
apikeys-tool-mail-read-desc = メッセージ本文を読みます。本文は信頼できない入力として明示されます。
apikeys-tool-folders-list-label = フォルダーを一覧表示
apikeys-tool-folders-list-desc = メールボックスのフォルダーを一覧表示します。
apikeys-tool-drafts-create-label = 下書きを作成
apikeys-tool-drafts-create-desc = 下書きメッセージを作成します（自動送信されることはありません）。
apikeys-tool-mail-send-label = メールを送信
apikeys-tool-mail-send-desc = アプリ内で確認するため、メッセージを送信トレイに入れます。
apikeys-tool-calendar-read-label = カレンダーを読む
apikeys-tool-calendar-read-desc = カレンダーの予定を読みます。
apikeys-tool-calendar-propose-label = 予定を提案
apikeys-tool-calendar-propose-desc = カレンダーの予定を提案します（確定はしません）。
apikeys-tool-tasks-read-label = タスクを読む
apikeys-tool-tasks-read-desc = タスクを読みます。
apikeys-tool-tasks-write-label = タスクを書き込む
apikeys-tool-tasks-write-desc = タスクを作成または更新します。
apikeys-tool-contacts-read-label = 連絡先を読む
apikeys-tool-contacts-read-desc = 連絡先を読みます。
apikeys-surface-scripts-label = Sieve スクリプトを管理
