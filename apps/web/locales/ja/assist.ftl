# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: assist. Source: ../en/assist.ftl

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = AI アシスタント
assist-heading = AI アシスタント
assist-transcript-label = AI アシスタントとの会話
assist-proposed-action = 提案された操作
assist-outbox-note = これにより、メッセージが送信トレイに入ります。そこで確認するまで何も送信されません。
assist-review-outbox = 送信トレイで確認
assist-review = 確認
assist-empty = AI アシスタントに要約、下書き、検索を依頼できます。操作を提案することはありますが、確定するのは常にあなた自身です。
assist-error = AI アシスタントは現在応答できません。
assist-input-label = AI アシスタントにメッセージを送る
assist-input-placeholder = AI アシスタントに質問…
assist-thinking = 考えています…
assist-ask = 質問

# -- Composer tools ---------------------------------------------------------
assist-composer-label = 作成支援
assist-composer-toolbar = 作成ツール
assist-tool-grammar = 文法を修正
assist-tool-rewrite = 書き直す
assist-tool-tone = トーンを調整
assist-tool-translate = 翻訳
assist-toolarg-tone = トーン
assist-toolarg-translate = 言語
assist-tone-neutral = 中立的
assist-tone-friendly = 親しみやすい
assist-tone-formal = フォーマル
assist-tone-concise = 簡潔
assist-tone-assertive = はっきりした
assist-busy = { $label }…
assist-composer-empty-error = 先に何か入力してください。
assist-composer-error = AI アシスタントは現在その処理を完了できません。
assist-suggested-edit = 提案された編集
assist-apply-draft = 下書きに適用
assist-discard = 破棄

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = 提案されたラベル
assist-apply-auto = 自動的に適用
assist-apply-label = { $label } を適用
assist-remove-label = { $label } を削除
assist-apply = 適用
assist-undo = 元に戻す

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = 長押しで音声入力
assist-dictate-stop = 音声入力を停止
assist-dictate-hold-text = 🎙 長押しで音声入力
assist-dictate-listening = ● 聞き取り中…
assist-dictate-endpoint-note = 音声は AI アシスタントのエンドポイントで文字起こしされます。
assist-dictate-err = 音声入力のエラーです。
assist-transcribe-err = 音声を文字起こしできませんでした。
assist-mic-err = マイクを使用できません。

# -- Semantic search --------------------------------------------------------
assist-semantic-label = セマンティック検索
assist-semantic-note = 意味に基づいて順位付けするため、検索語は AI アシスタントのエンドポイントに送信されます。

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = このデバイスから送信される可能性があるもの
assist-left-1 = 選択した機能のために選んだメッセージのテキスト（件名と本文）
assist-left-2 = 送信先のエンドポイントのホスト
assist-left-3 = 送信しない: エンドツーエンド暗号化されたコンテンツ（既定で除外）
assist-left-4 = 送信しない: 添付ファイル（既定で除外）
assist-left-5 = 送信しない: あなたの認証情報や他のアカウント
assist-disclosure-off = AI アシスタントはオフです。メッセージの内容はこのデバイスから送信されません。
assist-disclosure-sentence = AI アシスタントのツールを使用すると、選択したメッセージのテキストが { $host } に中継されます。{ $excl } 送信が自動で行われることはありません。送信トレイから何かが出ていく前に、必ずあなたが確認します。
assist-disclosure-excl-one = { $a }は決して送信しません。
assist-disclosure-excl-two = { $a }や{ $b }は決して送信しません。
assist-disclosure-admin-allowed = 管理者が、暗号化されたコンテンツと添付ファイルの送信を許可しています。
assist-excluded-e2ee = エンドツーエンド暗号化されたコンテンツ
assist-excluded-attachments = 添付ファイル

# -- Admin → AI assistant (deployment configuration) ------------------------------
assist-admin-title = AI アシスタント
assist-admin-intro = AI アシスタントは、ユーザーが選択したメッセージのテキストを、ここで設定した AI エンドポイントに送信します。メールの送信、削除、受け入れはできません。エンドツーエンド暗号化されたコンテンツと添付ファイルは、下で許可しない限り送信されません。
assist-admin-load-unreadable = 保存されている AI アシスタントの設定を読み取れません: API キーをこのサーバーの鍵（MW_SERVER_KEY）で開けません。別の鍵で保存されたか、変更されています。AI アシスタントはオフです。API キーを含めてフォームに入力し、保存すると置き換えられます。
assist-admin-load-error = AI アシスタントの設定を読み込めませんでした。保存済みの設定が上書きされないよう、保存は無効になっています。
assist-admin-status-running = AI アシスタントは実行中です。リクエストは { $host } に送られます。
assist-admin-status-running-pending = AI アシスタントは、サーバーの起動時の設定で実行中です。保存した変更はサーバーの再起動時に有効になります。
assist-admin-status-off = AI アシスタントはオフです。AI アシスタントへのリクエストはすべて拒否されます。
assist-admin-status-pending = AI アシスタントは実行されていません。保存した設定はサーバーの再起動時に有効になります。
assist-admin-status-local-program-not-allowed = AI アシスタントは実行されていません。設定されたプログラム { $program } はサーバーの MW_ASSIST_LOCAL_PROGRAMS に含まれていないため、サーバーはこれを起動しませんでした。
assist-admin-status-no-endpoint = AI アシスタントはオンですが、使用可能なエンドポイントが設定されていないため実行されていません。
assist-admin-stop = AI アシスタントを今すぐ停止
assist-admin-stop-note = 停止はすべてのユーザーに対してすぐに有効になり、再起動後も維持されます。
assist-admin-resume = AI アシスタントをオンにする
assist-admin-resume-note = サーバーが AI アシスタントをオフにした状態で起動した場合、オンにする操作はサーバーの再起動時に有効になります。
assist-admin-kill-error = AI アシスタントのオン・オフを変更できませんでした。
assist-admin-endpoint = エンドポイント
assist-admin-endpoint-note = このエンドポイントに接続するのはサーバーで、ブラウザーが接続することはありません。変更はサーバーの再起動時に有効になります。
assist-admin-kind = エンドポイントの種類
assist-admin-kind-none = なし
assist-admin-kind-open-ai-compatible = OpenAI 互換
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = ローカルプログラム
assist-admin-local-program-enable = AI アシスタントにこのサーバー上のプログラムを実行させるには、サーバーを起動する環境で MW_ASSIST_LOCAL_PROGRAMS にプログラムの絶対パスを設定し（複数のパスはカンマで区切ります）、サーバーを再起動してください。
assist-admin-local-program-note = サーバーは AI アシスタントへのリクエストごとにこのプログラムを起動し、ユーザーが選択したメッセージのテキストを含むリクエストを標準入力で渡します。応答はプログラムの標準出力から読み取ります。選択できるのは、サーバーの MW_ASSIST_LOCAL_PROGRAMS に含まれるプログラムだけです。
assist-admin-local-program = プログラム
assist-admin-local-program-unlisted = { $program }（リストにない）
assist-admin-local-program-unlisted-note = このプログラムは MW_ASSIST_LOCAL_PROGRAMS に含まれていないため、サーバーは実行せず、保存時にも受け付けません。リストにあるプログラムか、別のエンドポイントの種類を選んでください。
assist-admin-local-program-required = 実行するプログラムを選んでください。
assist-admin-local-args = 引数（1 行に 1 つ）
assist-admin-local-args-note = 各行が 1 つの引数としてプログラムに渡されます。シェルは使用されません。
assist-admin-base-url = ベース URL
assist-admin-base-url-required = エンドポイントのベース URL を http:// または https:// から入力してください。
assist-admin-api-key = API キー
assist-admin-api-key-stored = キーが保存されています。サーバーはキーを再表示しません。
assist-admin-api-key-replace = API キーを置き換える
assist-admin-api-key-replace-placeholder = 保存済みのキーを維持するには空欄のままにします
assist-admin-api-key-remove = 保存済みのキーを削除
assist-admin-api-key-keep = 保存済みのキーを維持
assist-admin-api-key-remove-pending = 保存すると、保存済みのキーは削除されます。
assist-admin-chat-model = チャットモデル
assist-admin-embed-model = 埋め込みモデル
assist-admin-stt-model = 音声認識モデル
assist-admin-model = モデル
assist-admin-anthropic-version = API バージョン
assist-admin-max-tokens = 応答の最大トークン数
assist-admin-default-placeholder = サーバーの既定値
assist-admin-grants = 機能
assist-admin-grants-note = 使用できるのは、ここでチェックした機能だけです。
assist-admin-cap-summarize = 要約
assist-admin-cap-draft = 下書きと書き直し
assist-admin-cap-grammar = 文法
assist-admin-cap-dictation = 音声入力
assist-admin-cap-search-semantic = セマンティック検索
assist-admin-cap-auto-tag = 自動タグ付け
assist-admin-cap-recap = 振り返り
assist-admin-cap-assistant = AI アシスタントとのチャット
assist-admin-ceilings = データの制限
assist-admin-ceilings-note = これらの制限は、機能にかかわらずすべてのリクエストに適用されます。
assist-admin-accounts = メールの送信を許可するアカウント ID（1 行に 1 つ）
assist-admin-accounts-note = アカウントが 1 つも指定されていない場合、メールの内容はエンドポイントに送信されません。
assist-admin-folders = 送信を限定するフォルダー ID（1 行に 1 つ）
assist-admin-folders-note = フォルダーが 1 つも指定されていない場合、指定したアカウントのすべてのフォルダーが許可されます。
assist-admin-allow-e2ee = エンドツーエンド暗号化されたコンテンツの送信を許可する
assist-admin-allow-attachments = 添付ファイルの送信を許可する
assist-admin-save = 設定を保存
assist-admin-saved = 保存しました。
assist-admin-saved-pending = 保存しました。変更はサーバーの再起動時に有効になります。
assist-admin-save-error = 設定は保存されませんでした。
assist-admin-save-not-allowed = サーバーがこのプログラムを許可していないため、設定は保存されませんでした。使用できるのは MW_ASSIST_LOCAL_PROGRAMS に含まれるプログラムだけです。
