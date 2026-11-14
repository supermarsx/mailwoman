# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-settings. Source: ../en/admin-settings.ftl
admin-settings-key-mail-allow-send = 送信を許可する
admin-settings-help-mail-allow-send = ユーザーがメッセージを送信できるようにします。オフにすると、作成画面で送信とそのショートカットが無効になり、サーバーは送信を拒否します。下書きは引き続き保存できます。
admin-settings-key-mail-allow-reply = 返信を許可する
admin-settings-help-mail-allow-reply = ユーザーがリーダーやショートカットから返信と全員に返信を行えるようにします。オフにすると、これらの操作は表示されません。
admin-settings-key-mail-allow-forward = 転送を許可する
admin-settings-help-mail-allow-forward = ユーザーがメッセージを本文内または添付ファイルとして転送できるようにします。オフにすると、これらの操作は表示されません。
admin-settings-key-mail-allow-download = ダウンロードを許可する
admin-settings-help-mail-allow-download = ユーザーがメッセージと添付ファイルをダウンロードできるようにします。オフにすると、Web アプリはダウンロードを表示せず、サーバーはメッセージのエクスポートを拒否します。
admin-settings-key-mail-allow-print = 印刷を許可する
admin-settings-help-mail-allow-print = ユーザーがリーダーからメッセージを印刷できるようにします。オフにすると、印刷の操作は表示されません。
admin-settings-key-mail-show-spam = スパムフォルダーを表示する
admin-settings-help-mail-show-spam = スパムまたは迷惑メールフォルダーと、スパムとしてマークされたメッセージを表示します。オフにすると、フォルダー一覧と検索結果で非表示になります。何も削除されません。
admin-settings-key-mail-show-auth-status = 送信者認証の結果を表示する
admin-settings-help-mail-show-auth-status = リーダーに Authentication-Results ヘッダーの SPF、DKIM、DMARC の結果を表示します。
admin-settings-key-mail-close-on-delete = 削除したメッセージを閉じる
admin-settings-help-mail-close-on-delete = 開いているメッセージが削除またはゴミ箱に移動されたとき、そのメッセージのタブまたは別ウィンドウを閉じます。オフにすると、リーダーは従来の動作のままです。
admin-settings-key-links-trusted-senders = 信頼するリンク送信者
admin-settings-help-links-trusted-senders = 確認済みメッセージのリンクをメッセージごとの同意なしで開ける、完全一致の送信者メールボックス。1 行に 1 アドレス。
admin-settings-key-appearance-font-size = インターフェイスのフォントサイズ
admin-settings-help-appearance-font-size = インターフェイスのフォントサイズ（ピクセル）。12 から 24 まで。
admin-settings-key-appearance-font-weight = インターフェイスのフォントの太さ
admin-settings-help-appearance-font-weight = インターフェイスのテキストの基本の太さ：300、400、500、600。強調されたテキストは階層を保ちます。
admin-settings-key-appearance-font-color = インターフェイスの文字色
admin-settings-help-appearance-font-color = インターフェイスの文字色を #RRGGBB で指定します。空欄にするとテーマに従います。
admin-settings-key-appearance-low-contrast = 文字のコントラストを弱める
admin-settings-help-appearance-low-contrast = インターフェイスの文字のコントラストを弱めます。
admin-settings-key-mail-message-list-order = メッセージ一覧の並び順
admin-settings-help-mail-message-list-order = メッセージ一覧の並び順：新しい順、古い順、送信者、件名。
admin-settings-key-mail-message-list-style = メッセージ一覧のスタイル
admin-settings-help-mail-message-list-style = メッセージ一覧をカード形式、または列見出し付きの行形式で表示します。
admin-settings-key-mail-message-list-preview = メッセージ一覧のプレビュー
admin-settings-help-mail-message-list-preview = メッセージ一覧に本文の抜粋を表示します。
admin-settings-key-mail-after-delete = メッセージを削除した後
admin-settings-help-mail-after-delete = 開いているメッセージを削除した後にリーダーが表示するメッセージ：次、前、なし。
admin-settings-key-reader-attachments-at-end = 添付ファイルを本文の後に表示
admin-settings-help-reader-attachments-at-end = 添付ファイルを本文の前ではなく後に表示します。
admin-settings-key-notifications-ask-permission = 通知の許可を案内する
admin-settings-help-notifications-ask-permission = 通知を許可もブロックもしていないデバイスのユーザーに、ブラウザーの通知許可プロンプトを案内します。

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-title = 設定
admin-settings-description = サーバー全体、ドメインごと、ユーザーごとの値です。ロックされた値は、ユーザーやより狭い範囲では変更できません。
admin-settings-list-caption = 設定
admin-settings-search = 設定を検索
admin-settings-empty = 検索に一致する設定はありません。
admin-settings-unavailable = このサーバーはまだ設定モデルに対応していません。
admin-settings-col-setting = 設定
admin-settings-col-value = 値
admin-settings-col-source = 取得元
admin-settings-col-overrides = より狭い範囲の値
admin-settings-open = 開く
admin-settings-overrides-none = なし
admin-settings-overrides = { $domains } ドメイン、{ $users } ユーザー
admin-settings-not-found = { $key } という設定はありません。
admin-settings-value-on = オン
admin-settings-value-off = オフ
admin-settings-value-empty = 未設定
admin-settings-value-list = { $count ->
   *[other] { $count } 件
}
admin-settings-detail-key = キー: { $key }
admin-settings-global = サーバー全体
admin-settings-global-help = ドメインまたはユーザーにより狭い範囲の値がある場合、または許可されている範囲でユーザーが自分の値を選んだ場合を除き、この値がすべてのユーザーに適用されます。
admin-settings-value-label = 値
admin-settings-json-hint = JSON。サーバーが値を検証し、受け付けない箇所を示します。
admin-settings-json-invalid = 有効な JSON ではありません。
admin-settings-user-override = ユーザーによる変更を許可
admin-settings-user-override-help = オンの場合、ユーザーは自分の設定で独自の値を選べます。オフの場合は、ここで設定した値またはドメインの値が適用されます。
admin-settings-lock-help = ロックすると、ドメインの値とユーザーの選択は無視され、この値が全員に適用されます。
admin-settings-save = 保存
admin-settings-saved = 保存しました。すべてのサーバープロセスで 30 秒以内に反映されます。
admin-settings-saved-restart = 保存しました。再起動後に反映されます。
admin-settings-dedicated = この設定は { $section } で変更します。影響を先に確認できます。
admin-settings-dedicated-link = { $section } を開く
admin-settings-pinned = 環境変数 { $name } で設定されています。この変数が未設定の場合は、ここに保存された値が適用されます。
admin-settings-lock-title = { $setting } をロックしますか？
admin-settings-lock-body = ロックすると、下に一覧表示されたより狭い範囲の値は無視されます。値は保持され、ロックを解除すると再び適用されます。
admin-settings-lock-confirm = ロック
admin-settings-impact-domain = { $count ->
   *[other] { $count } 件のドメイン値が無視されます
}
admin-settings-impact-user = { $count ->
   *[other] { $count } 人のユーザー自身の選択が無視されます
}
admin-settings-impact-admin-user = { $count ->
   *[other] { $count } 件のユーザー別の値が無視されます
}
admin-settings-impact-profile = { $count ->
   *[other] { $count } 件の機能プロファイルの値が無視されます
}
admin-settings-impact-group = { $count ->
   *[other] { $count } 件のグループの値が無視されます
}
admin-settings-domains = ドメインの値
admin-settings-domains-help = ドメインの値は、サーバー全体の値の代わりに、そのドメインのユーザーに適用されます。
admin-settings-domains-caption = { $setting } のドメインの値
admin-settings-domains-empty = 独自の値を持つドメインはありません。
admin-settings-domain = ドメイン
admin-settings-domain-value = このドメインの値
admin-settings-domain-locked = ドメインのユーザーに対してロック
admin-settings-domain-add = ドメインの値を追加
admin-settings-domain-save = ドメインの値を保存
admin-settings-domain-remove = 削除
admin-settings-domain-remove-title = { $domain } の値を削除しますか？
admin-settings-domain-remove-body = { $domain } のユーザーには、再びサーバー全体の値が適用されます。
admin-settings-domain-required = ドメイン名を入力してください。
admin-settings-locked-yes = ロック済み
admin-settings-locked-no = ロックなし
admin-settings-users = ユーザー別の値
admin-settings-users-help = 管理者が特定のユーザーに設定した値です。上限の場合、ユーザーはアドレスで指定します。
admin-settings-users-caption = { $setting } のユーザー別の値
admin-settings-users-empty = 管理者が値を設定したユーザーはいません。
admin-settings-user = ユーザー
admin-settings-user-hint = アカウント ID。上限の場合はアドレス。
admin-settings-user-value = このユーザーの値
admin-settings-user-add = ユーザー別の値を追加
admin-settings-user-save = ユーザーの値を保存
admin-settings-user-required = ユーザーを入力してください。
admin-settings-user-remove-title = { $user } の値を削除しますか？
admin-settings-user-remove-body = このユーザーには、ドメインの値またはサーバー全体の値が適用されます。
admin-settings-source-default = 組み込みの既定値
admin-settings-source-envFallback = 環境変数（保存された値なし）
admin-settings-source-envPin = 環境変数
admin-settings-source-global = サーバー全体に保存
admin-settings-cat-interface = インターフェイス
admin-settings-cat-appearance = 外観
admin-settings-cat-mail = メール
admin-settings-cat-folders = フォルダーのレイアウト
admin-settings-cat-notifications = 通知
admin-settings-cat-offline = オフライン利用
admin-settings-cat-assistant = AI アシスタント
admin-settings-cat-upstream = メールサーバー
admin-settings-cat-signin = サインイン
admin-settings-cat-admin = 管理コンソール
admin-settings-cat-passwords = パスワード
admin-settings-cat-limits = 上限と使用量
admin-settings-cat-plugins = プラグイン
admin-settings-cat-features = 機能の権限
admin-settings-cat-integrations = 連携の権限
admin-settings-key-cap-integration-nextcloud = Nextcloud
admin-settings-help-cap-integration-nextcloud = アカウントにリンクされた Nextcloud のファイル、共有、およびカレンダー/連絡先/タスクの操作を許可します。
admin-settings-key-cap-integration-google = Google ブリッジ
admin-settings-help-cap-integration-google = トークンの取得と更新を含む Google ブリッジの操作を許可します。
admin-settings-key-cap-integration-microsoft = Microsoft ブリッジ
admin-settings-help-cap-integration-microsoft = トークンの取得と更新を含む Microsoft ブリッジの操作を許可します。
admin-settings-key-cap-integration-family = 個別の連携
admin-settings-help-cap-integration-family = 個別の連携識別子に対する権限です。実行時の対象範囲は機能カタログに記載されています。
admin-settings-key-cap-ui-plugin-family = 個別の UI プラグイン
admin-settings-help-cap-ui-plugin-family = 1 つの UI プラグインに対する権限です。連携カタログにあるポリシーキーをそのまま使用してください。
admin-settings-key-cap-ai-assistant = AI アシスタント
admin-settings-help-cap-ai-assistant = アシスタントの設定と呼び出しを許可します。
admin-settings-key-cap-plugins = プラグイン
admin-settings-help-cap-plugins = UI プラグインとエンジンプラグインの操作を許可します。
admin-settings-key-cap-pim-calendar = カレンダー
admin-settings-help-cap-pim-calendar = カレンダーのメソッドと購読を許可します。
admin-settings-key-cap-pim-contacts = 連絡先
admin-settings-help-cap-pim-contacts = 連絡先とアドレス帳のメソッドを許可します。
admin-settings-key-cap-pim-tasks = タスク
admin-settings-help-cap-pim-tasks = タスクのメソッドを許可します。
admin-settings-key-cap-pim-notes = メモ
admin-settings-help-cap-pim-notes = メモのメソッドを許可します。
admin-settings-key-cap-export = エクスポート
admin-settings-help-cap-export = メールと PIM のエクスポートを許可します。
admin-settings-key-cap-import = インポート
admin-settings-help-cap-import = メールと PIM のインポートを許可します。
admin-settings-key-cap-api-keys = API キー
admin-settings-help-cap-api-keys = API キーのポリシーです。適用範囲は機能カタログを確認してください。
admin-settings-key-cap-oauth-apps = OAuth アプリケーション
admin-settings-help-cap-oauth-apps = OAuth アプリケーションのポリシーです。適用範囲は機能カタログを確認してください。
admin-settings-key-cap-rules = メールルール
admin-settings-help-cap-rules = 管理対象のメールルールと自動応答のメソッドを許可します。
admin-settings-key-cap-rules-forward = ルールによる転送
admin-settings-help-cap-rules-forward = 転送ポリシーです。適用範囲は機能カタログを確認してください。
admin-settings-key-cap-remote-images = リモート画像
admin-settings-help-cap-remote-images = リモート画像の権限です。リクエストは引き続き既存のプライバシー設定に従います。機能の対象範囲を確認してください。
admin-settings-key-cap-user-background = カスタム背景
admin-settings-help-cap-user-background = カスタム背景の権限です。既定ではオフです。機能の対象範囲を確認してください。
admin-settings-key-cap-user-css = カスタム CSS
admin-settings-help-cap-user-css = カスタム CSS の権限です。既定ではオフです。機能の対象範囲を確認してください。
admin-settings-key-limit-attachment-bytes = 添付ファイルのサイズ上限
admin-settings-help-limit-attachment-bytes = バイト単位の添付ファイルポリシーで、既定は 50000000 です。実行時の適用は未実装です。
admin-settings-key-limit-send-per-hour = 1 時間あたりの送信上限
admin-settings-help-limit-send-per-hour = 1 時間あたりの送信ポリシーです。0 は無制限を意味します。実行時の適用は未実装です。
admin-settings-key-limit-recipients-per-message = 1 通あたりの宛先数
admin-settings-help-limit-recipients-per-message = 宛先数のポリシーです。0 は無制限を意味します。実行時の適用は未実装です。
admin-settings-cat-import = インポート
admin-settings-cat-webhooks = Webhook
admin-settings-cat-realtime = リアルタイム更新
admin-settings-cat-privacy = プライバシー
admin-settings-cat-observability = オブザーバビリティ
admin-settings-cat-egress = 外向きの接続
admin-settings-cat-updates = アップデート
admin-settings-cat-backups = バックアップ
admin-settings-cat-about = 情報
admin-settings-cat-config = 構成
admin-settings-cat-jobs = ジョブ
admin-settings-cat-maintenance = メンテナンス
admin-settings-cat-health = ヘルスチェック
admin-settings-cat-diagnostics = 診断
admin-settings-cat-users = ユーザー
admin-settings-cat-server-text = サーバーのテキスト
admin-settings-key-ui-language = 言語
admin-settings-help-ui-language = 言語を選択していないユーザー向けのメールクライアントの言語です。「ブラウザーと同じ」は各ブラウザーの言語に従います。
admin-settings-key-ui-notify-new-version = 新バージョンの通知
admin-settings-help-ui-notify-new-version = オン（既定）の場合、ページの読み込み後に新しいバージョンの Web アプリが配備されると、サインイン中のユーザーに「今すぐ再読み込み」と「後で」のトーストを表示します。オフの場合は表示せず、ユーザーは次回の再読み込み時に新しいコードを取得します。
admin-settings-key-ui-undo-history-size = 元に戻す/やり直しの履歴サイズ
admin-settings-help-ui-undo-history-size = 「元に戻す」と「やり直し」のボタンが記憶する最近のメール操作（アーカイブ、削除、移動、既読にする、フラグ、ラベルなど）の数で、0 から 50 までです。0 にするとボタンと Ctrl+Z ショートカットがオフになります。既定は 50 です。管理者がロックしない限り、ユーザーは下の最大値を超えない範囲で自分のサイズを選べます。
admin-settings-key-folders-reorder-enabled = フォルダーのドラッグ＆ドロップ
admin-settings-help-folders-reorder-enabled = サイドバーと設定で、フォルダーをドラッグして並べ替えられるようにします。オフにすると、ドラッグハンドルと移動コマンドが非表示になります。これがロックされていない限り、ユーザーは自分用にオフにできます。
admin-settings-key-folders-reorder-allow-nesting = フォルダーの入れ子
admin-settings-help-folders-reorder-allow-nesting = フォルダーを別のフォルダーにドロップして入れ子にしたり、最上位に戻したりできるようにします。オフの場合、各フォルダーは現在の親の下に留まり、同じ階層内での並べ替えのみ可能です。
admin-settings-key-folders-reorder-allow-server-move = サーバー上でフォルダーを移動
admin-settings-help-folders-reorder-allow-server-move = 入れ子にする、または解除すると、メールサーバー上のフォルダーの実際の親が変わり、あらゆる場所で変更が反映されます。オフの場合、これらの移動はこのアプリ内だけの配置として保持されます。
admin-settings-key-folders-reorder-allow-system-folders = システムフォルダーを移動
admin-settings-help-folders-reorder-allow-system-folders = 受信トレイ、下書き、送信済み、迷惑メール、ゴミ箱、アーカイブをドラッグして並べ替えられるようにします。オフの場合は上部に固定されます。並べ替えはこのアプリ内だけで行われ、サーバー上で名前の変更や移動は行われません。
admin-settings-key-folders-reorder-max-depth = フォルダーの入れ子の深さ
admin-settings-help-folders-reorder-max-depth = ドラッグで作成できるフォルダーの入れ子の最大の深さで、1 から 16 までです。フォルダーまたはそのサブフォルダーがこれより深くなる移動は拒否されます。
admin-settings-key-folders-order-locked = フォルダーの順序を固定
admin-settings-help-folders-order-locked = 対象範囲の全員のフォルダーの順序を固定します。ドラッグと移動コマンドはオフになり、ユーザーにはロックのヒントが表示されます。ピン留めと管理者が設定した順序はそのまま残ります。
admin-settings-key-folders-pinned-admin = 強制ピン留めフォルダー
admin-settings-help-folders-pinned-admin = すべてのユーザーのサイドバー上部の「ピン留め」グループに表示するフォルダーです。1 行に 1 件、role:inbox または Shared/Support のようなフォルダーパスで指定します。ユーザーはピン留めの解除や移動ができません。一致するフォルダーがない項目はスキップされます。
admin-settings-key-folders-order-default = 既定のフォルダー順序
admin-settings-help-folders-order-default = フォルダーを並べ替えていないユーザーの初期の並び順で、先頭から末尾の順です。1 行に 1 件、role:<role> またはフォルダーパスで指定します。一覧にないフォルダーは、サーバーの順序で後に続きます。ユーザー自身の順序がこれに優先します。
admin-settings-key-folders-new-folder-placement = 新しいフォルダーの配置
admin-settings-help-folders-new-folder-placement = サーバー上に新しく現れたフォルダーを同じ階層のどこに置くか。末尾、またはアルファベット順です。
admin-settings-key-ui-undo-history-max = 元に戻す/やり直し履歴の最大数
admin-settings-help-ui-undo-history-max = 「元に戻す」と「やり直し」の履歴にユーザーが保持できる操作の最大数で、0 から 50 までです。ユーザー自身のサイズはこの値まで下げられます。0 にすると全員の履歴がオフになります。履歴はブラウザーのタブ内に保持され、サインアウト時に消去されます。
admin-settings-key-ui-bar-mail-top-bar = メールツールバー
admin-settings-help-ui-bar-mail-top-bar = 「pinned」はレイアウトのコントロールを含むメールツールバーを常に表示します。「auto」は細いハンドルの後ろに隠し、ポインターを合わせる、タップする、または Tab キーで移動すると開きます。
admin-settings-key-ui-bar-workspace-tabs = ワークスペースのタブ
admin-settings-help-ui-bar-workspace-tabs = 「pinned」はワークスペースのタブバーを常に表示します。「auto」は、ユーザーが開くまで、アクティブなタブの名前だけを表示する細いハンドルを表示します。
admin-settings-key-ui-bar-mail-ribbon = メールのリボン
admin-settings-help-ui-bar-mail-ribbon = リボンレイアウト用です。「pinned」はリボンを開いたままにします。「auto」はタブ名だけを表示し、タブを開くとメール一覧の上にリボンが表示されます。
admin-settings-key-ui-bar-composer-ribbon = エディターのリボン
admin-settings-help-ui-bar-composer-ribbon = すべての作成画面用です。「pinned」はエディターのリボンを開いたままにします。「auto」はタブ名だけを表示し、タブを開くとメッセージ本文が覆われずに下へ押し下げられます。
admin-settings-key-appearance-theme = テーマ
admin-settings-help-appearance-theme = ユーザーが自分で選ぶまで表示されるカラーテーマです。ロックすると、すべてのユーザーがこのテーマになります。
admin-settings-key-appearance-accent = アクセントカラー
admin-settings-help-appearance-accent = テーマのアクセントカラーです。空の場合はテーマ自身のアクセントを使用します。
admin-settings-key-appearance-badge-color = 未読バッジの色
admin-settings-help-appearance-badge-color = 未読数バッジの色です。default はテーマの色を維持し、accent はアクセントカラーに従います。accent-lighter と accent-darker はアクセントを 20% 明るく、または暗くした色を使い、数字は黒または白になります。ロックはアカウントの選択より優先されます。
admin-settings-key-appearance-density = 密度
admin-settings-help-appearance-density = リストやコントロールが占める余白の大きさ。コンパクト、標準、ゆったりのいずれかです。
admin-settings-key-appearance-font = インターフェイスのフォント
admin-settings-key-packages-allowed = 荷物追跡を許可
admin-settings-help-packages-allowed = 対象のアカウントで荷物追跡を許可します。荷物追跡でプロバイダーも有効にし、各ユーザーが利用を選択する必要があります。この設定を無効にすると、新しいプロバイダー呼び出しは直ちに行われなくなります。
admin-settings-help-appearance-font = フォントの設定がないアカウント向けに組み込みフォントを選ぶか、対象のすべてのアカウントでその選択をロックします。フォントはこのサーバーから読み込まれ、Google Fonts へのリクエストは行われません。既定はテーマのフォントを使用します。
admin-settings-key-appearance-brand-name = 製品名
admin-settings-help-appearance-brand-name = サインインページとブラウザーのタブに表示される名前です。
admin-settings-key-login-notice = サインインのお知らせ
admin-settings-help-login-notice = サインインページに表示する短いテキストです。たとえばサポートの連絡先など。空にすると何も表示しません。
admin-settings-key-mail-read-receipts = 開封確認
admin-settings-key-receipts-dsn-allowed = 配信状況のリクエスト
admin-settings-help-receipts-dsn-allowed = アカウントが SMTP の配信状況通知を要求できるようにします。メッセージが読まれたことを要求したり証明したりするものではありません。
admin-settings-help-mail-read-receipts = 開いたメッセージが開封確認を求めたときの動作です。ユーザーに確認する、送信しない、常に送信する、のいずれかです。
admin-settings-key-notifications-enabled = 新着メールの通知
admin-settings-help-notifications-enabled = ユーザーが新着メールの通知を受け取るかどうかです。通知ルールは変更されません。
admin-settings-key-offline-retention-days = オフラインキャッシュの保持期間
admin-settings-help-offline-retention-days = デバイスがオフライン用にメッセージを保持する日数です。0 は期限なしを意味します。
admin-settings-key-offline-budget-mb = オフラインキャッシュのサイズ
admin-settings-help-offline-budget-mb = デバイスがオフラインメールに使用できるメガバイト数です。0 はオフラインキャッシュなしを意味します。
admin-settings-key-assistant-enabled = AI アシスタント
admin-settings-help-assistant-enabled = ユーザーに AI アシスタントを表示するかどうかです。AI アシスタントのデプロイスイッチが、これより上位で引き続き適用されます。
admin-settings-key-assist-profile = AI プロバイダープロファイル
admin-settings-help-assist-profile = これらのユーザーに対応するプロバイダープロファイルの ID です（AI アシスタント > プロバイダープロファイル）。空の場合はデプロイのエンドポイントを使用します。存在しないプロファイル ID を指定すると、これらのユーザーの AI アシスタントはオフになります。
admin-settings-key-upstream-default = 既定のメールサーバー
admin-settings-help-upstream-default = どのドメインもメールサーバーを指定していない場合に、アドレスがサインインするメールサーバーです。ドメインの表がこれより優先されます。
admin-settings-key-upstream-proxy-timeout-secs = メールサーバーのタイムアウト
admin-settings-help-upstream-proxy-timeout-secs = メールサーバーへの 1 回のリクエストにかけられる時間（秒）です。
admin-settings-key-signin-new-users = 新規ユーザー
admin-settings-help-signin-new-users = 初回サインイン時にアカウントを作成できる人：誰でも（open）、事前に追加されたアドレスのみ（preprovisioned）、それに加えてインポートされたユーザー（provisioned）、または新規は不可（closed）。既存のアカウントには影響しません。サインインポリシーで変更します。
admin-settings-key-signin-new-users-mail = パスワードサインイン：新規ユーザー
admin-settings-help-signin-new-users-mail = 新しいメールボックスのユーザーがパスワード（IMAP、POP3、Exchange）でサインインする場合のルールです。「継承」は「新規ユーザー」を使用します。
admin-settings-key-signin-new-users-sso = シングルサインオン：新規ユーザー
admin-settings-help-signin-new-users-sso = 新しいユーザーがシングルサインオンまたは信頼されたヘッダーでサインインする場合のルールです。「継承」は「新規ユーザー」を使用します。
admin-settings-key-signin-new-users-linked = リンクされたメールボックス：新規ユーザー
admin-settings-help-signin-new-users-linked = ここにまだアカウントがないメールボックスをリンクする場合のルールで、リンクされたメールボックス自身のドメインで判断します。「継承」は「新規ユーザー」を使用します。
admin-settings-key-signin-new-users-linked-imap = リンクされた IMAP メールボックス：新規ユーザー
admin-settings-help-signin-new-users-linked-imap = リンクされた IMAP メールボックスのみのルールです。「継承」は「リンクされたメールボックス」を使用します。
admin-settings-key-signin-new-users-linked-pop3 = リンクされた POP3 メールボックス：新規ユーザー
admin-settings-help-signin-new-users-linked-pop3 = リンクされた POP3 メールボックスのみのルールです。「継承」は「リンクされたメールボックス」を使用します。
admin-settings-key-signin-new-users-linked-ews = リンクされた Exchange メールボックス：新規ユーザー
admin-settings-help-signin-new-users-linked-ews = リンクされた Exchange（EWS）メールボックスのみのルールです。「継承」は「リンクされたメールボックス」を使用します。
admin-settings-key-signin-preprovision-ttl-days = 事前追加ユーザーの有効期間（日）
admin-settings-help-signin-preprovision-ttl-days = ほかの有効期間が指定されていない場合に、事前に追加された保留中ユーザーが有効な期間です。
admin-settings-key-signin-preprovision-max-ttl-days = 事前追加ユーザーの最長有効期間（日）
admin-settings-help-signin-preprovision-max-ttl-days = 管理者が、事前に追加された保留中ユーザーに設定できる最長の有効期間です。
admin-settings-key-signin-preprovision-domain-wide = ドメイン全体の事前追加エントリーを許可
admin-settings-help-signin-preprovision-domain-wide = 使用回数の上限付きで、ドメイン全体の誰でも受け入れるエントリーを許可します。既定ではオフです。
admin-settings-key-signin-preprovision-retention-days = 期限切れの保留中ユーザーを保持（日）
admin-settings-help-signin-preprovision-retention-days = 期限切れの保留中ユーザーを削除するまで保持する期間です。
admin-settings-key-signin-domain-mode = サインインできるドメイン
admin-settings-help-signin-domain-mode = どのドメインでもサインインできるか、一覧にあるドメインのみかです。サインインポリシーで変更します。
admin-settings-key-signin-refusal-detail = 拒否メッセージ
admin-settings-help-signin-refusal-detail = 拒否されたサインインで理由（無効なアカウント、許可されていないアドレス）を伝えるか、パスワードが違う場合と同じ応答にするかです。サインインポリシーで変更します。
admin-settings-key-signin-ldap-bind-precheck = サインイン前の LDAP チェック
admin-settings-help-signin-ldap-bind-precheck = メールサーバーより先に、ディレクトリでパスワードを確認します。テストバインドの後、連携で変更します。
admin-settings-key-twofa-pending-ttl-secs = 2 要素目の制限時間
admin-settings-help-twofa-pending-ttl-secs = パスワード入力後に 2 要素目を入力できる時間（秒）です。
admin-settings-key-twofa-max-attempts = 2 要素目の試行回数
admin-settings-help-twofa-max-attempts = 2 要素目のコードを誤って入力できる回数で、超えるとサインインが最初からやり直しになります。
admin-settings-key-admin-session-idle-secs = コンソールのアイドルタイムアウト
admin-settings-help-admin-session-idle-secs = リクエストがない状態で管理コンソールのセッションが続く時間（秒）です。
admin-settings-key-admin-session-absolute-secs = コンソールのセッション上限
admin-settings-help-admin-session-absolute-secs = 操作の有無にかかわらず、管理コンソールのセッションが続く最長時間（秒）です。
admin-settings-key-admin-session-rotate-secs = コンソールのセッション ID のローテーション
admin-settings-help-admin-session-rotate-secs = 使用中に管理コンソールがセッション ID を入れ替える間隔（秒）です（0 = ステップアップ確認の後のみ）。ローテーションによってアイドル期限や絶対期限が延長されることはありません。
admin-settings-key-passwd-policy-min-length = パスワードの最小文字数
admin-settings-help-passwd-policy-min-length = 新しいパスワードに必要な最小の文字数です。
admin-settings-key-passwd-policy-require-upper = 大文字を必須にする
admin-settings-help-passwd-policy-require-upper = 新しいパスワードには大文字を含める必要があります。
admin-settings-key-passwd-policy-require-lower = 小文字を必須にする
admin-settings-help-passwd-policy-require-lower = 新しいパスワードには小文字を含める必要があります。
admin-settings-key-passwd-policy-require-digit = 数字を必須にする
admin-settings-help-passwd-policy-require-digit = 新しいパスワードには数字を含める必要があります。
admin-settings-key-passwd-policy-require-symbol = 記号を必須にする
admin-settings-help-passwd-policy-require-symbol = 新しいパスワードには、文字でも数字でもない文字を含める必要があります。
admin-settings-key-limit-storage-bytes = ストレージの上限
admin-settings-help-limit-storage-bytes = 1 人のユーザーがこのサーバーに保存できる最大のバイト数です。0 は無制限を意味します。
admin-settings-key-limit-send-per-day = 1 日あたりの送信数
admin-settings-help-limit-send-per-day = 1 人のユーザーが 1 日に送信できる最大のメッセージ数です。0 は無制限を意味します。
admin-settings-key-limit-ai-calls-per-day = 1 日あたりの AI アシスタント呼び出し数
admin-settings-help-limit-ai-calls-per-day = 1 人のユーザーが 1 日に行える AI アシスタントのリクエストの最大数です。0 は無制限を意味します。
admin-settings-key-quota-warn-thresholds = 使用量の警告レベル
admin-settings-help-quota-warn-thresholds = ユーザーに警告する上限に対する割合で、低い順に指定します。
admin-settings-key-quota-warn-enabled = 使用量の警告
admin-settings-help-quota-warn-enabled = 上限に近づいたときにユーザーへ警告するかどうかです。
admin-settings-key-usage-retention-days = 使用量の履歴
admin-settings-help-usage-retention-days = 日ごとの使用量カウントを保持する日数です。
admin-settings-key-usage-meter-api = API の使用をカウント
admin-settings-help-usage-meter-api = API リクエストを使用量の数値に含めるかどうかです。
admin-settings-key-usage-measure-interval-hours = ストレージ測定の間隔
admin-settings-help-usage-measure-interval-hours = 各ユーザーの保存データを測定する頻度（時間）です。
admin-settings-key-assistant-rate-limit-per-min = AI アシスタントのレート制限
admin-settings-help-assistant-rate-limit-per-min = サーバー全体で 1 分あたりに受け付ける AI アシスタントのリクエストの最大数です。0 は無制限を意味します。
admin-settings-key-plugins-kv-max-key-bytes = プラグインのキーサイズ
admin-settings-help-plugins-kv-max-key-bytes = プラグインが保存できる最長のキー（バイト）です。
admin-settings-key-plugins-kv-max-value-bytes = プラグインの値のサイズ
admin-settings-help-plugins-kv-max-value-bytes = プラグインが 1 つのキーに保存できる最大の値（バイト）です。
admin-settings-key-plugins-kv-max-total-bytes = プラグインのストレージ
admin-settings-help-plugins-kv-max-total-bytes = 1 つのプラグインが保存できる合計の最大バイト数です。
admin-settings-key-plugins-kv-max-keys = プラグインのキー数
admin-settings-help-plugins-kv-max-keys = 1 つのプラグインが保存できるキーの最大数です。
admin-settings-key-import-max-messages = インポートするメッセージ数
admin-settings-help-import-max-messages = 1 回のインポートで取り込めるメッセージの最大数です。
admin-settings-key-import-max-message-bytes = インポートするメッセージのサイズ
admin-settings-help-import-max-message-bytes = インポートで受け付ける最大のメッセージ（バイト）です。
admin-settings-key-webhooks-max-attempts = Webhook の試行回数
admin-settings-help-webhooks-max-attempts = Webhook の配信を諦めるまでに試行する回数です。
admin-settings-key-webhooks-timeout-secs = Webhook のタイムアウト
admin-settings-help-webhooks-timeout-secs = 1 回の Webhook 配信にかけられる時間（秒）です。
admin-settings-key-outbox-max-attempts = 送信の試行回数
admin-settings-help-outbox-max-attempts = メッセージが未送信として報告されるまでに、メールサーバーへ渡される回数です。
admin-settings-key-realtime-recheck-secs = リアルタイム更新の再確認
admin-settings-help-realtime-recheck-secs = 開いているリアルタイム更新の接続が、セッションがまだ有効かを確認する間隔（秒）です。
admin-settings-key-privacy-watermark-enabled = 閲覧者の透かし
admin-settings-help-privacy-watermark-enabled = 開いたメッセージの上に、閲覧者のアドレスを入れた透かしを表示します。
admin-settings-key-privacy-watermark-opacity = 透かしの濃さ
admin-settings-help-privacy-watermark-opacity = 透かしの見えやすさで、0.02（薄い）から 0.5 までです。
admin-settings-key-privacy-open-tracking-retention-days = 開封追跡の履歴
admin-settings-help-privacy-open-tracking-retention-days = 既読追跡の記録を保持する日数です。
admin-settings-key-privacy-open-tracking-unbound-ttl-hours = 未送信の追跡トークン
admin-settings-help-privacy-open-tracking-unbound-ttl-hours = 追跡トークンが、メッセージの送信を待って削除されるまでの時間数です。
admin-settings-key-masked-email-domain = マスクされたアドレスのドメイン

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-masked-email-domain = マスクされたアドレスを作成するドメインです。空にするとマスクされたアドレスはオフになります。
admin-settings-key-abuse-report-address = 不正利用の報告先アドレス
admin-settings-help-abuse-report-address = 迷惑メールや不正利用の報告の送信先です。
admin-settings-key-dlp-admin-address = データ漏えい通知
admin-settings-help-dlp-admin-address = データ漏えい防止の通知の送信先です。
admin-settings-key-observability-error-forward-url = エラー転送先アドレス
admin-settings-help-observability-error-forward-url = クライアントのエラーレポートを転送する https アドレスです。空の場合は転送しません。
admin-settings-key-egress-probe-url = 外向き接続のテスト用アドレス
admin-settings-help-egress-probe-url = 外向きプロキシのテストに使用する https アドレスです。
admin-settings-key-push-vapid-contact = プッシュの連絡先
admin-settings-help-push-vapid-contact = 各通知とともにプッシュサービスへ渡される連絡先（mailto: または https:）です。
admin-settings-key-push-quiet-hours = プッシュの静音時間帯
admin-settings-help-push-quiet-hours = プッシュ通知を送信しない時間帯（UTC）です。空の場合はありません。
admin-settings-key-updates-check = アップデートの確認
admin-settings-help-updates-check = サーバーが 1 日に 1 回、新しいリリースを確認し、ダッシュボードに表示するかどうかです。
admin-settings-key-updates-feed-url = リリースフィード
admin-settings-help-updates-feed-url = アップデートの確認で読み取る https アドレスです。
admin-settings-key-updates-apply = アップデートの適用
admin-settings-help-updates-apply = ホストのアップデートエージェントを通じて、コンソールからアップデートを開始できるかどうかです。
admin-settings-key-backup-retention-count = 保持するバックアップ数
admin-settings-help-backup-retention-count = 保持するデータベースバックアップの数です。
admin-settings-key-about-security-contact = セキュリティの連絡先
admin-settings-help-about-security-contact = security.txt に記載する連絡先（mailto: または https:）です。
admin-settings-key-about-security-policy-url = セキュリティポリシー
admin-settings-help-about-security-policy-url = security.txt に記載する、セキュリティポリシーのアドレスです。
admin-settings-key-about-project-links = 情報ページのリンク
admin-settings-help-about-project-links = 情報ページに表示するリンクで、それぞれラベルと https アドレスを持ちます。
admin-settings-key-config-snapshots-kept = 構成スナップショット
admin-settings-help-config-snapshots-kept = 保持する構成スナップショットの数です。
admin-settings-key-jobs-history-days = ジョブの履歴
admin-settings-help-jobs-history-days = ジョブの実行記録を保持する日数です。
admin-settings-key-maintenance-announce-minutes-default = メンテナンス通知の事前時間
admin-settings-help-maintenance-announce-minutes-default = メンテナンス時間帯の何分前にユーザーへ知らせるかです。
admin-settings-key-maintenance-default-message = メンテナンスのメッセージ
admin-settings-help-maintenance-default-message = メンテナンス中にユーザーへ表示するテキストで、言語ごとに設定します。
admin-settings-key-health-disk-free-min-percent = ディスク空き容量の下限
admin-settings-help-health-disk-free-min-percent = ディスクの空き容量がこの割合を下回ると、レディネスチェックが失敗します。
admin-settings-key-observability-otlp-metrics = OTLP でメトリクスを送信
admin-settings-help-observability-otlp-metrics = 起動時に設定された OTLP コレクターへサーバーのメトリクスを送信します。
admin-settings-key-metrics-drilldown-top-n = メトリクス詳細の行数
admin-settings-help-metrics-drilldown-top-n = メトリクスの内訳に表示する行数です。
admin-settings-key-metrics-rollup-days = メトリクスの履歴
admin-settings-help-metrics-rollup-days = 1 時間ごとのメトリクス集計を保持する日数です。
admin-settings-key-observability-client-timing-enabled = ブラウザーの読み込み時間
admin-settings-help-observability-client-timing-enabled = サインイン中のブラウザーからページの読み込み時間を、匿名の集計ヒストグラムとして収集します。個人、セッション、ページを特定するものは含まれません。オフにすると収集を停止します。
admin-settings-key-observability-client-timing-sample-rate-percent = ブラウザー読み込み時間のサンプリング率
admin-settings-help-observability-client-timing-sample-rate-percent = 読み込み時間を報告するブラウザーセッションの割合です（1 から 100）。
admin-settings-key-observability-client-timing-min-samples = 読み込み時間の最小サンプル数
admin-settings-help-observability-client-timing-min-samples = 期間のサンプル数がこの数に達するまで、読み込み時間の中央値と 95 パーセンタイルは非表示のままです。
admin-settings-key-observability-client-timing-folder-loads = フォルダーの読み込み時間
admin-settings-help-observability-client-timing-folder-loads = フォルダーがメッセージ一覧を表示するまでの時間を、ブラウザーキャッシュから来たかネットワークから来たかで分けて測定します。既定ではオンです。この設定がロックされていない限り、ユーザーは自分用にオフにできます。ブラウザーの読み込み時間がオンである必要があります。
admin-settings-key-observability-client-timing-message-loads = メッセージの読み込み時間
admin-settings-help-observability-client-timing-message-loads = メッセージがヘッダーとサニタイズ済みの本文を表示するまでの時間を、キャッシュ済み、先読み済み、ネットワークから取得のいずれかで分けて測定します。既定ではオンです。この設定がロックされていない限り、ユーザーは自分用にオフにできます。ブラウザーの読み込み時間がオンである必要があります。
admin-settings-key-diagnostics-retention-days = 診断の履歴
admin-settings-help-diagnostics-retention-days = 診断の実行記録を保持する日数です。
admin-settings-key-data-export-link-hours = データエクスポートのダウンロード期間
admin-settings-help-data-export-link-hours = データエクスポートが削除されるまでにダウンロードできる時間数です。
admin-settings-key-dsr-enabled = データリクエスト
admin-settings-help-dsr-enabled = 設定から、データに関するリクエスト（アクセス、ダウンロード、訂正、消去、制限、異議）を提出できるかどうかです。
admin-settings-key-dsr-type-access = アクセスリクエストを受け付ける
admin-settings-help-dsr-type-access = アクセスリクエストの種類をユーザーに提供するかどうかです。
admin-settings-key-dsr-type-portability = ダウンロード（ポータブルコピー）リクエストを受け付ける
admin-settings-help-dsr-type-portability = ダウンロード（ポータブルコピー）リクエストの種類をユーザーに提供するかどうかです。
admin-settings-key-dsr-type-rectification = 訂正リクエストを受け付ける
admin-settings-help-dsr-type-rectification = 訂正リクエストの種類をユーザーに提供するかどうかです。
admin-settings-key-dsr-type-erasure = 消去リクエストを受け付ける
admin-settings-help-dsr-type-erasure = 消去リクエストの種類をユーザーに提供するかどうかです。
admin-settings-key-dsr-type-restriction = 制限リクエストを受け付ける
admin-settings-help-dsr-type-restriction = 制限リクエストの種類をユーザーに提供するかどうかです。
admin-settings-key-dsr-type-objection = 異議リクエストを受け付ける
admin-settings-help-dsr-type-objection = 異議リクエストの種類をユーザーに提供するかどうかです。
admin-settings-key-dsr-type-other = その他のリクエストを受け付ける
admin-settings-help-dsr-type-other = 汎用のリクエストの種類をユーザーに提供するかどうかです。
admin-settings-key-dsr-approval-export = アクセスおよびダウンロードリクエストの承認
admin-settings-help-dsr-approval-export = auto はリクエストが確認されるとエクスポートを開始します。admin は管理者の承認を待ちます。
admin-settings-key-dsr-approval-erasure = 消去リクエストの承認
admin-settings-help-dsr-approval-erasure = auto はリクエストが確認されると消去をスケジュールします。admin は管理者の承認を待ちます。
admin-settings-key-dsr-approval-other = その他のリクエストの処理
admin-settings-help-dsr-approval-other = auto は確認されたリクエストを対応中にします。admin は管理者の承認を待ちます。
admin-settings-key-dsr-reauth-required = リクエストの再認証
admin-settings-help-dsr-reauth-required = エクスポートまたは消去のリクエストを提出するとき、およびダウンロードリンクを作成するときに、パスワードまたは 2 要素目の再入力を求めます。
admin-settings-key-dsr-confirmation-required = 確認メッセージ
admin-settings-help-dsr-confirmation-required = コード付きの確認メッセージをリクエストした人の受信トレイに送信します。コードが入力されるまでリクエストは進みません。
admin-settings-key-dsr-identity-verification-required = 本人確認の手順
admin-settings-help-dsr-identity-verification-required = リクエストを承認する前に、管理者がリクエストした人の本人確認を行ったことを記録する必要があります。
admin-settings-key-dsr-erasure-grace-days = 消去の猶予期間
admin-settings-help-dsr-erasure-grace-days = 承認から消去までの日数です。この間、リクエストした人は取り消せます。0 の場合、次のワーカー処理で消去を実行します。
admin-settings-key-dsr-sla-days = リクエストへの回答までの日数
admin-settings-help-dsr-sla-days = 提出から、ユーザーと管理者に表示される期限までの日数です。
admin-settings-key-dsr-sla-max-extension-days = 最長の延長
admin-settings-help-dsr-sla-max-extension-days = 管理者が理由を記録したうえで、リクエストの期限に追加できる最大の日数です。
admin-settings-key-dsr-sla-reminder-days = 期限のフラグ
admin-settings-help-dsr-sla-reminder-days = 期限の何日前から、キュー内でリクエストに「まもなく期限」のフラグを付けるかです。
admin-settings-key-dsr-export-mail-allowed = エクスポートに含めるメール
admin-settings-help-dsr-export-mail-allowed = リクエストした人が、エクスポートにメッセージを含められるようにします。含まれるのはこのサーバー自身のキャッシュにあるメッセージのみで、アップストリームのメールサーバーに残っているメールは含まれません。
admin-settings-key-dsr-export-mail-format = エクスポートのメール形式
admin-settings-help-dsr-export-mail-format = mbox はフォルダーごとに 1 ファイル、eml はメッセージごとに 1 ファイルを書き出します。
admin-settings-key-dsr-export-include-settings = エクスポート：設定
admin-settings-help-dsr-export-include-settings = エクスポートに設定を含めます。
admin-settings-key-dsr-export-include-identities = エクスポート：差出人情報
admin-settings-help-dsr-export-include-identities = エクスポートに差出人情報を含めます。
admin-settings-key-dsr-export-include-contacts = エクスポート：連絡先
admin-settings-help-dsr-export-include-contacts = エクスポートに連絡先を含めます。
admin-settings-key-dsr-export-include-calendars = エクスポート：カレンダー
admin-settings-help-dsr-export-include-calendars = エクスポートにカレンダーを含めます。
admin-settings-key-dsr-export-include-notes = エクスポート：メモ
admin-settings-help-dsr-export-include-notes = エクスポートにメモを含めます。
admin-settings-key-dsr-export-include-rules = エクスポート：フィルターとルール
admin-settings-help-dsr-export-include-rules = エクスポートにフィルターとルールを含めます。
admin-settings-key-dsr-export-include-sessions = エクスポート：セッション、デバイス、サインイン履歴
admin-settings-help-dsr-export-include-sessions = エクスポートにセッション、デバイス、サインイン履歴を含めます。
admin-settings-key-dsr-export-include-consent = エクスポート：同意の記録
admin-settings-help-dsr-export-include-consent = エクスポートに同意の記録を含めます。
admin-settings-key-dsr-export-include-audit = エクスポート：本人に関する監査エントリー
admin-settings-help-dsr-export-include-audit = エクスポートに本人に関する監査エントリーを含めます。
admin-settings-key-dsr-export-max-megabytes = エクスポートのサイズ上限
admin-settings-help-dsr-export-max-megabytes = エクスポートの最大サイズ（メガバイト）です。上限を超えるセクションは除外され、マニフェストに記載されます。
admin-settings-key-dsr-export-format = エクスポート形式
admin-settings-help-dsr-export-format = json は JSON ファイルを書き出します。jsonCsv は一覧データについて CSV ファイルも書き出します。
admin-settings-key-dsr-export-encrypt = 保存時にエクスポートを暗号化
admin-settings-help-dsr-export-encrypt = 作成されたエクスポートは、ダウンロードされるか期限切れになるまで、サーバーのキーで暗号化して保存します。
admin-settings-key-dsr-download-link-minutes = ダウンロードリンクの有効期間
admin-settings-help-dsr-download-link-minutes = ダウンロードリンクが作成されてから使用できる分数です。エクスポート自体は、データエクスポートのダウンロード期間の間、保持されます。
admin-settings-key-dsr-download-single-use = ダウンロードリンクを 1 回限りにする
admin-settings-help-dsr-download-single-use = ダウンロードリンクは、1 回使用されると無効になります。
admin-settings-key-dsr-retention-completed-days = 完了したリクエストの保持期間
admin-settings-help-dsr-retention-completed-days = 完了したリクエストとその履歴を削除するまで保持する日数です。
admin-settings-key-dsr-notify-admins = 新しいリクエストを管理者向けに記録
admin-settings-help-dsr-notify-admins = 新しいリクエストを、管理者とアラートの送信先が確認できる監査ログに、セキュリティイベントとして記録します。
admin-settings-key-dsr-notify-requester = リクエストした人への進捗メッセージ
admin-settings-help-dsr-notify-requester = リクエストの状態が変わったときに、リクエストした人の受信トレイにメッセージを入れます。
admin-settings-key-dsr-contact-name = データ保護の連絡先の名前
admin-settings-help-dsr-contact-name = データリクエストのページでユーザーに表示される名前です。
admin-settings-key-dsr-contact-email = データ保護の連絡先アドレス
admin-settings-help-dsr-contact-email = データリクエストのページでユーザーに表示されるメールアドレスです。
admin-settings-key-dsr-contact-url = データ保護の連絡先ページ
admin-settings-help-dsr-contact-url = データリクエストのページでユーザーに表示されるリンクです。
admin-settings-key-dsr-on-behalf-allowed = ユーザーに代わるリクエスト
admin-settings-help-dsr-on-behalf-allowed = 郵送で受け取ったリクエストなど、管理者がユーザーに代わってリクエストを提出できるようにします。
admin-settings-key-dsr-rate-limit-per-day = 1 人あたりの 1 日のリクエスト数
admin-settings-help-dsr-rate-limit-per-day = 1 人が 24 時間以内に提出できるリクエストの最大数です。
admin-settings-key-dsr-legal-hold-subjects = 法的保全
admin-settings-help-dsr-legal-hold-subjects = 消去が禁止されるアドレス、または @domain エントリーです。これらに対する消去リクエストは、エントリーが削除されるまでキュー内でブロック状態のまま保持されます。
admin-settings-key-scrub-enabled = 個人情報のスクラブ
admin-settings-help-scrub-enabled = 「個人情報をスクラブ」操作と、消去の後に続けて行えるスクラブを許可します。オフにすると、どちらも実行されません。
admin-settings-key-scrub-pseudonym-algorithm = 仮名の長さ
admin-settings-help-scrub-pseudonym-algorithm = 識別子を置き換えるキー付き仮名の作り方です。現在のキー世代による HMAC-SHA256 を、16 または 32 桁の 16 進文字に切り詰めます。
admin-settings-key-scrub-include-logs = ログファイルもスクラブ
admin-settings-help-scrub-include-logs = スクラブでは、ローテーション済みのログファイル（圧縮されたものを含む）内の識別子も書き換えます。
admin-settings-key-scrub-auto-run-on-erasure = 消去後にスクラブ
admin-settings-help-scrub-auto-run-on-erasure = 消去が完了すると、監査履歴とログにある本人の識別子も仮名に置き換えます。既定ではオフです。
admin-settings-key-scrub-key-retention-days = 仮名キーの保持期間
admin-settings-help-scrub-key-retention-days = ローテーションで使われなくなった仮名キーを保持する日数です。0 の場合は保持し続けます。削除後は、古い仮名を再計算できなくなり、そのキーで作られたトゥームストーンを再生できなくなります。
admin-settings-key-backup-propagate-erasure = 復元時に消去を再適用
admin-settings-help-backup-propagate-erasure = 復元時に、バックアップより新しい消去のトゥームストーンを再適用し、その後に消去された人が再び削除されるようにします。
admin-settings-key-backup-tombstone-retention-days = 消去のトゥームストーンの保持期間
admin-settings-help-backup-tombstone-retention-days = 消去のトゥームストーンを保持する日数です。0 の場合は永久に保持します。
admin-settings-key-dsr-text-intro = データリクエストの案内文
admin-settings-help-dsr-text-intro = リクエストフォームの上に表示されるテキストです。言語ごとに 1 つの値を設定します。
admin-settings-key-dsr-text-confirmation = 確認メッセージの文面
admin-settings-help-dsr-text-confirmation = 確認メッセージに追加されるテキストです。言語ごとに 1 つの値を設定します。
admin-settings-key-dsr-text-completion = 完了メッセージの文面
admin-settings-help-dsr-text-completion = リクエストの完了時に送信されるメッセージに追加されるテキストです。言語ごとに 1 つの値を設定します。
admin-settings-key-server-text-family = サーバーのテキスト
admin-settings-help-server-text-family = サーバーがメールや通知に書き込むテキストで、言語とドメインごとに設定します。
admin-settings-cat-sieve = メールフィルター
admin-settings-key-sieve-max-script-bytes = フィルタースクリプトの最大サイズ
admin-settings-help-sieve-max-script-bytes = 1 つの Sieve フィルタースクリプトの最大バイト数です。
admin-settings-key-sieve-max-scripts = フィルタースクリプトの最大数
admin-settings-help-sieve-max-scripts = アカウントごとに保存できるスクリプトの最大数です。
admin-settings-key-sieve-max-redirects = リダイレクトの最大数
admin-settings-help-sieve-max-redirects = 1 つのスクリプト内のリダイレクトコマンドの最大数です。
admin-settings-key-sieve-vacation-allowed = 不在通知のフィルター動作
admin-settings-help-sieve-vacation-allowed = バックエンドが対応している場合に、スクリプトが不在通知の返信を送信できるようにします。
admin-settings-key-sieve-allowed-extensions = 許可するフィルター拡張
admin-settings-help-sieve-allowed-extensions = 小文字の Sieve 拡張名の任意のリストです。バックエンドも各拡張に対応している必要があります。
admin-settings-key-cap-sieve-raw-edit = フィルタースクリプトのソースを編集
admin-settings-help-cap-sieve-raw-edit = 選択したアカウントで、Sieve スクリプトを直接編集できるようにします。
admin-settings-key-cap-sieve-forward-external = アカウントのドメイン外への転送
admin-settings-help-cap-sieve-forward-external = スクリプトでの外部への転送を許可します。これらのスクリプトの保存には、最近の本人確認も必要です。
admin-settings-key-session-max-concurrent = 同時セッション数
admin-settings-help-session-max-concurrent = アカウントごとのセッションの最大数です。新しいサインインでこの上限を超えると、最も古いセッションが無効になります。
admin-settings-key-session-idle-secs = セッションのアイドルタイムアウト
admin-settings-help-session-idle-secs = アイドル状態のセッションを、この秒数の後に終了します。デプロイの上限までです。
admin-settings-key-session-absolute-secs = セッションの有効期間
admin-settings-help-session-absolute-secs = セッションを、この秒数の後に終了します。デプロイの上限までです。
admin-settings-key-twofa-remember-days = 記憶したデバイスの有効期間
admin-settings-help-twofa-remember-days = 記憶したデバイスが、再び 2 要素目の確認を必要とするまでの日数です。0 の場合、この機能は無効になります。
admin-settings-key-twofa-passkeys-enabled = パスキーを許可
admin-settings-help-twofa-passkeys-enabled = パスキーを認証要素として使用できるようにします。
admin-settings-key-twofa-required = 2 要素認証を必須にする
admin-settings-help-twofa-required = セッションを発行する前に、登録済みの 2 要素目を必須にします。
admin-settings-key-twofa-passkey-attachment = パスキーの認証器の種類
admin-settings-help-twofa-passkey-attachment = any（既定）：すべての認証器。platform：デバイスに保存されるパスキー（Windows Hello、Touch ID、Android の画面ロック）。cross-platform：セキュリティキーとスマートフォン。これはブラウザーが報告するもので、認証器による署名はありません。
admin-settings-key-twofa-passkey-user-verification = パスキーのユーザー検証
admin-settings-help-twofa-passkey-user-verification = required：パスキーは追加時とサインインのたびに PIN または生体認証を確認する必要があり、それがない手続きは拒否されます。preferred（既定）と discouraged は、ブラウザーに求める内容を変えるだけです。
admin-settings-key-twofa-passkey-resident-key = 検出可能なパスキー
admin-settings-help-twofa-passkey-resident-key = required：認証情報を認証器に保存する必要があります。discouraged（既定）または preferred。エンジンモードでは、パスワードなしでもサインインできるため、パスキーは常に検出可能です。
admin-settings-key-twofa-passkey-legacy = 現在のポリシーより前に追加されたパスキー
admin-settings-help-twofa-passkey-legacy = allow（既定）：ポリシーを満たさない古いパスキーも引き続き使用でき、ユーザーに交換を促します。refuse：サインイン時にメッセージとともに拒否され、ユーザーは別の要素またはリカバリーコードでサインインします。
admin-settings-key-twofa-last-factor-removal = 最後の 2 要素目の削除
admin-settings-help-twofa-last-factor-removal = パスキーまたは認証アプリを削除すると、アカウントに 2 要素目が 1 つもなくなる場合の動作です。allow（既定）：削除され、サインインにはパスワードだけが必要になります。block：ユーザーは先に別のサインイン方法を追加する必要があります（リカバリーコードは含まれません）。2 要素認証が必須の場合、最後の 1 つは常に保持されます。
admin-settings-key-privacy-remote-content = リモートコンテンツ
admin-settings-help-privacy-remote-content = 画像をブロックする、先に確認する、プロキシを使う、または許可されている場合は直接読み込みます。
admin-settings-key-privacy-trackers = トラッカーをブロック
admin-settings-help-privacy-trackers = 既知のメール追跡用画像をブロックします。
admin-settings-key-privacy-clean-links = リンクをクリーンアップ
admin-settings-help-privacy-clean-links = メッセージ内のリンクから追跡用パラメーターを削除します。
admin-settings-key-privacy-max-bytes = リモート画像のサイズ上限
admin-settings-help-privacy-max-bytes = 1 つのリモート画像の取得に許可される最大バイト数です。
admin-settings-key-privacy-cache-bytes = リモート画像のキャッシュ
admin-settings-help-privacy-cache-bytes = リモート画像のキャッシュに保持される最大バイト数です。
admin-settings-key-privacy-rate-per-min = リモート画像のリクエスト頻度
admin-settings-help-privacy-rate-per-min = アカウントごとに 1 分あたりに取得できるリモート画像の最大数です。
admin-settings-privacy-fetch-title = リモート画像の取得
admin-settings-privacy-fetch-help = サーバーが閲覧者のために取得する画像の、時間制限、リダイレクト、並列ダウンロードです。上限を超えた画像にはプレースホルダーが表示されます。
admin-settings-key-privacy-fetch-connect-timeout-ms = リモート画像の接続タイムアウト
admin-settings-help-privacy-fetch-connect-timeout-ms = TLS ハンドシェイクを含め、画像ホストへの接続を開くのに許可されるミリ秒数です。既定は 5000 です。
admin-settings-key-privacy-fetch-headers-timeout-ms = リモート画像の応答タイムアウト
admin-settings-help-privacy-fetch-headers-timeout-ms = リクエストの開始から、画像ホストがヘッダーで応答するまでに許可されるミリ秒数です。既定は 10000 です。
admin-settings-key-privacy-fetch-idle-timeout-ms = リモート画像の停止タイムアウト
admin-settings-help-privacy-fetch-idle-timeout-ms = 画像のダウンロード中、2 つのパート間で許される最長の停止時間（ミリ秒）で、超えると中止されます。既定は 10000 です。
admin-settings-key-privacy-fetch-total-timeout-ms = リモート画像の合計タイムアウト
admin-settings-help-privacy-fetch-total-timeout-ms = リダイレクトを含め、画像の取得に許可されるミリ秒数です。キューでの待機には別の予算があるため、混み合った画像が開始前にネットワーク時間を使い果たすことはありません。既定は 20000 です。
admin-settings-key-privacy-fetch-queue-timeout-ms = リモート画像のキュータイムアウト
admin-settings-help-privacy-fetch-queue-timeout-ms = 取得の予算に加えて、画像が空きを待てる最大のミリ秒数です。上限付きのキューにより、認証済みの大量の同時リクエストにも対応しつつ、実行中の取得とデコードの上限は引き続き適用されます。既定は 60000 です。
admin-settings-key-privacy-max-redirects = リモート画像のリダイレクト
admin-settings-help-privacy-max-redirects = 1 つの画像で追従するリダイレクトの回数です。それぞれ、最初のリクエストと同じアドレスのルールで確認されます。既定は 3 です。
admin-settings-key-privacy-max-concurrent-per-message = メッセージごとのリモート画像ダウンロード数
admin-settings-help-privacy-max-concurrent-per-message = 1 通のメッセージの画像を同時に取得する数です。残りは空きを待ちます。範囲は 1～128、既定は 30 です。
admin-settings-key-privacy-max-concurrent-per-account = アカウントごとのリモート画像ダウンロード数
admin-settings-help-privacy-max-concurrent-per-account = 1 つのアカウントで、開いているすべてのメッセージにわたって同時に取得する画像の数です。範囲は 1～256、既定は 128 です。
admin-settings-key-privacy-allow-direct = 画像の直接読み込みを許可
admin-settings-key-cap-remote-content-direct = 画像の直接読み込みの権限
admin-settings-help-cap-remote-content-direct = サーバーが許可している場合に、このアカウントが画像の直接読み込みを選択できるようにします。
admin-settings-help-privacy-allow-direct = 必要な権限を持つアカウントに対して、画像の直接読み込みを許可します。
admin-settings-key-logs-auth-retention-days = 認証履歴の保持期間
admin-settings-help-logs-auth-retention-days = 認証のメタデータを、永続的なアカウントの識別情報とは別に保持する日数です。
admin-settings-key-logs-auth-max-entries = 認証履歴のエントリー数の上限
admin-settings-help-logs-auth-max-entries = 保持するエントリーの最大数です。1 分以内に繰り返された同じ失敗は 1 つのエントリーにまとめられます。
admin-settings-key-reader-color-mode = メッセージの色
admin-settings-cat-reader = メッセージの表示
admin-settings-key-mail-row-actions-mode = メッセージ一覧のクイック操作
admin-settings-help-mail-row-actions-mode = 各メッセージ行のクイック操作の表示方法です。open（常に利用可能で、ホバーまたはフォーカス時に表示）、hover（3 つのコントロールと「その他」ボタン）、click（それらを開く 1 つのボタン）、off（行から削除。コンテキストメニューとツールバーにはすべての操作が残ります）。タッチ画面では hover は click と同じ動作になります。管理者がロックしない限り、ユーザーは自分で選べます。
admin-settings-key-mail-quick-reply = クイック返信
admin-settings-help-mail-quick-reply = 宛先があらかじめ入力された、コンパクトな返信タブを提供します。既定ではオフです。管理者がロックしない限り、ユーザーは有効にできます。通常の送信保護は引き続き適用されます。
admin-settings-key-mail-request-read-receipt = 既定で開封確認を要求
admin-settings-help-mail-request-read-receipt = 対応している送信メッセージで、最初から開封確認の要求を選択した状態にします。これは作成時の既定値であり、受信者が確認を返すことを保証するものではありません。
admin-settings-help-reader-color-mode = 自動はダークテーマに従います。分離されたメッセージフレーム内で、元のまま、適応、反転のいずれかの表示を選べます。
admin-settings-key-reader-font-size = メッセージのフォントサイズ
admin-settings-help-reader-font-size = 既定の閲覧サイズ（ピクセル）で、12 から 24 までです。
admin-settings-key-reader-line-height = メッセージの行の高さ
admin-settings-help-reader-line-height = 閲覧時の行間で、1.2 から 2.2 までです。
admin-settings-key-reader-content-width = メッセージの閲覧幅
admin-settings-help-reader-content-width = メッセージの内容の幅を、標準、広め、ペイン全体から選びます。
admin-settings-key-reader-loading = メッセージ読み込みの表示
admin-settings-help-reader-loading = プレースホルダーの行、テキスト表示、または視覚的な表示なしです。アクセシビリティ向けの読み上げは引き続き利用できます。
admin-settings-key-reader-loading-delay-ms = メッセージ読み込みの遅延
admin-settings-help-reader-loading-delay-ms = 読み込みの表示だけを 0 から 1000 ミリ秒の範囲で遅らせます。メッセージの配信が遅れることはありません。
admin-settings-key-reader-zoom = メッセージのズーム
admin-settings-help-reader-zoom = メッセージ本文の既定のズーム率（パーセント）で、50 から 200 までです。閲覧画面は最も近いズーム段階を使用します。拡大縮小されるのはメッセージのみで、印刷は 100% です。
admin-settings-key-reader-attachments-collapsed = 添付ファイルバーを折りたたむ
admin-settings-help-reader-attachments-collapsed = 閲覧画面の添付ファイルバーを、数と合計サイズだけを表示する折りたたみ状態で開始するかどうかです。ユーザーが最後に開いた、または閉じた状態がここに保存されます。既定ではオフで、添付ファイルが表示されます。
admin-settings-key-login-show-domain-hint = サインイン時に優先ドメインを表示
admin-settings-help-login-show-domain-hint = メールのサフィックスなしのユーザー名を受け付けるドメインを表示します。既定では非表示です。変わるのはヒントだけで、ユーザー名でのサインインは引き続き使えます。
admin-settings-key-signin-retry-cooldown-secs = サインイン再試行のクールダウン
admin-settings-help-signin-retry-cooldown-secs = 同じクライアントネットワークからのパスワードサインイン試行の最小間隔（秒）です。既定は 3 で、0 でクールダウンは無効になります。既存の失敗によるブロックは引き続き適用されます。
admin-settings-key-mail-quote-style = 返信と転送のヘッダー形式
admin-settings-key-mail-external-recipient-warning = 外部の宛先への警告
admin-settings-help-mail-external-recipient-warning = オフ、警告、または作成画面で確認を必須にします。宛先、Cc、Bcc が対象です。これは作成画面での安全策であり、サーバーの配信制限ではありません。
admin-settings-key-mail-trusted-recipient-domains = 信頼する宛先ドメイン
admin-settings-help-mail-trusted-recipient-domains = 完全一致する宛先ドメインを最大 100 件まで。選択した送信元の差出人情報のドメインも信頼されます。無効なエントリーは信頼されません。ユーザーによる上書きを防ぐには、この設定をロックします。
admin-settings-help-mail-quote-style = 従来の引用形式の返信、または Outlook 形式のメッセージヘッダーを使用します。これはユーザーの既定値を設定します。形式を強制するにはロックします。既存の下書きは変更されません。
admin-settings-key-signin-anti-bot-mode = サインインのボット対策
admin-settings-help-signin-anti-bot-mode = セルフホストのプルーフ・オブ・ワーク検証：off、mailbox、admin、all。外部の検証サービスには接続しません。
admin-settings-key-signin-anti-bot-difficulty = サインイン検証の作業量
admin-settings-help-signin-anti-bot-difficulty = ソルバーカウンターの最大値で、100～20000 です。値が大きいほどブラウザーの処理が増えます。既定は 2000 です。
admin-settings-key-signin-anti-bot-expiry-secs = サインイン検証の有効期間
admin-settings-help-signin-anti-bot-expiry-secs = 1 回限りの検証が有効な秒数で、60～600、既定は 180 です。認証情報の試行ごとに、新しい検証が必要です。
admin-settings-key-attachments-preview-allowed = 添付ファイルのプレビューを許可
admin-settings-help-attachments-preview-allowed = オフにすると、添付ファイルのプレビュー表示を防ぎます。元のファイルのダウンロードには別の設定があります。
admin-settings-key-attachments-download-allowed = 元の添付ファイルの提供を許可
admin-settings-help-attachments-download-allowed = オフにすると、元のバイト列を必要とするプレビューも無効になります。ブラウザーのプレビューはコピー防止ではありません。
admin-settings-key-attachments-max-bytes = 添付ファイルの最大バイト数
admin-settings-help-attachments-max-bytes = 0 は無制限です。範囲リクエストを含め、提供される元のバイト列に適用されます。
admin-settings-key-attachments-allowed-extensions = 許可するファイル名の拡張子
admin-settings-help-attachments-allowed-extensions = ドットなしの小文字の拡張子の JSON 配列です（例：["pdf","tar.gz"]）。空の場合はどの拡張子も許可します。ブロックルールが優先されます。
admin-settings-key-attachments-blocked-extensions = ブロックするファイル名の拡張子
admin-settings-help-attachments-blocked-extensions = 小文字の拡張子の JSON 配列です。保存された MIME のファイル名で判定し、ダウンロード URL の名前は使用しません。
admin-settings-key-attachments-warn-extensions = 警告するファイル名の拡張子
admin-settings-help-attachments-warn-extensions = 元のファイルを提供する前に、明示的な確認が必要な拡張子の JSON 配列です。
admin-settings-key-attachments-allowed-mime-types = 許可する MIME タイプ
admin-settings-help-attachments-allowed-mime-types = 小文字のタイプまたはトップレベルのワイルドカードの JSON 配列です（例：["image/*","application/pdf"]）。空の場合はどのタイプも許可します。
admin-settings-key-attachments-blocked-mime-types = ブロックする MIME タイプ
admin-settings-help-attachments-blocked-mime-types = 小文字のタイプまたはトップレベルのワイルドカードの JSON 配列です。ブロックルールが優先されます。
admin-settings-key-attachments-warn-mime-types = 警告する MIME タイプ
admin-settings-help-attachments-warn-mime-types = 確認が必要な小文字のタイプまたはトップレベルのワイルドカードの JSON 配列です。MIME のメタデータはマルウェア検出ではありません。
admin-settings-cat-shortcuts = キーボードショートカット
admin-settings-cat-time = タイムゾーン
admin-settings-key-time-display-zone = 表示するタイムゾーン
admin-settings-help-time-display-zone = Europe/Lisbon のような IANA タイムゾーン、または現在のデバイスに従う local です。メッセージのタイムスタンプは元の時刻のまま保存されます。
admin-settings-key-time-comparison-zone = 比較用のタイムゾーン
admin-settings-help-time-comparison-zone = メッセージとカレンダーの補助表示用に、任意で指定する 2 つ目の IANA タイムゾーンです。空にすると比較を非表示にします。メールアドレスから場所を推測することはありません。
admin-settings-key-time-show-helpers = タイムゾーンの補助表示
admin-settings-help-time-show-helpers = メッセージのタイムスタンプとカレンダーの予定について、現地時間と選択したタイムゾーンの時刻を並べて表示します。管理者のロックでこの選択を強制できます。
admin-settings-key-shortcuts-enabled = キーボードショートカットを有効にする
admin-settings-help-shortcuts-enabled = アプリのショートカットを許可します。ロックにより、全体またはドメイン単位で無効にできます。入力欄とモーダルダイアログでは、通常のキーボード処理が維持されます。
admin-settings-key-shortcuts-compose = 作成のショートカット
admin-settings-help-shortcuts-compose = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-archive = アーカイブのショートカット
admin-settings-help-shortcuts-archive = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-reply = 返信のショートカット
admin-settings-help-shortcuts-reply = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-reply-all = 全員に返信のショートカット
admin-settings-help-shortcuts-reply-all = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-forward = 転送のショートカット
admin-settings-help-shortcuts-forward = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-next = 次のメッセージのショートカット
admin-settings-help-shortcuts-next = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-previous = 前のメッセージのショートカット
admin-settings-help-shortcuts-previous = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-search = 検索にフォーカスのショートカット
admin-settings-help-shortcuts-search = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-refresh = メッセージを更新のショートカット
admin-settings-help-shortcuts-refresh = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-settings = 設定を開くのショートカット
admin-settings-help-shortcuts-settings = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-trash = ゴミ箱に移動のショートカット
admin-settings-help-shortcuts-trash = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-mark-read = 既読にするのショートカット

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-shortcuts-mark-read = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-mark-unread = 未読にするのショートカット
admin-settings-help-shortcuts-mark-unread = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-spam = 迷惑メールにするのショートカット
admin-settings-help-shortcuts-spam = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-zoom-in = メッセージを拡大のショートカット
admin-settings-help-shortcuts-zoom-in = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-zoom-out = メッセージを縮小のショートカット
admin-settings-help-shortcuts-zoom-out = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-zoom-reset = メッセージのズームをリセットのショートカット
admin-settings-help-shortcuts-zoom-reset = 選択したプリセットに従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-shortcuts-open-in-tab = メッセージを専用タブで開くのショートカット
admin-settings-help-shortcuts-open-in-tab = 選択したプリセット（Shift+Enter）に従うには inherit、この操作を削除するには disabled、または Ctrl+Shift+K のようなキーの組み合わせを指定します。競合する、または無効な組み合わせは実行されません。
admin-settings-key-notifications-push-allowed = バックグラウンドのプッシュ通知を許可
admin-settings-help-notifications-push-allowed = このアカウントで有効にした通知が、バックグラウンドのプッシュを登録して受信できるようにします。ブラウザーの許可は引き続き必要です。
admin-settings-key-offline-persistent-storage-allowed = ブラウザーの永続ストレージのリクエストを許可
admin-settings-help-offline-persistent-storage-allowed = ローカルにキャッシュされたデータをブラウザーの自動削除から保護するための、明示的なリクエストを許可します。ブラウザーが拒否する場合があり、これによって自動的に許可が与えられるわけではありません。
admin-settings-key-security-signin-mailbox-failure-limit = メールボックスの認証失敗の上限
admin-settings-help-security-signin-mailbox-failure-limit = カウント期間内に同じ送信元 IP から拒否された認証情報が上限に達すると、メールボックスのサインインが一時的にロックされます。ネットワークや構成の障害はカウントされません。
admin-settings-key-security-signin-admin-failure-limit = 管理者の認証失敗の上限
admin-settings-help-security-signin-admin-failure-limit = 同じ送信元 IP から管理者の認証情報が拒否されると、別途、管理者のサインインが一時的にロックされます。短い管理者セッションと最新の確認要件は変わりません。
admin-settings-key-security-signin-failure-window-secs = 認証失敗のカウント期間
admin-settings-help-security-signin-failure-window-secs = 認証情報および多要素認証の拒否に対するスライディング期間（秒）です。サインインが正常に完了すると、その送信元と対象はリセットされます。
admin-settings-key-security-signin-lock-secs = 一時的なサインインロックの期間
admin-settings-help-security-signin-lock-secs = ロックの期間（秒）です。ロック中の繰り返しの試行で、元の有効期限が延びることはありません。手動のグローバル IP ブロックとは別です。
admin-settings-key-security-signin-geo-scope = サインインの国ポリシーの対象範囲
admin-settings-help-security-signin-geo-scope = 既定ではオフです。新しいメールボックス、管理者、またはすべてのサインインにのみ適用します。既存のセッションは取り消されません。有効にする前に、現在の送信元の診断を確認してください。ローカルの GeoIP データベースが利用できない場合、サインインは拒否されます。
admin-settings-key-security-signin-geo-mode = 国の許可リストまたは拒否リスト
admin-settings-help-security-signin-geo-mode = allow は一覧にある国のみを許可し、deny は一覧にある国を拒否します。不明なアドレスには、それ専用の明示的なポリシーが使われます。
admin-settings-key-security-signin-geo-countries = サインインの国コード
admin-settings-help-security-signin-geo-countries = 大文字 2 文字の国コードの JSON 配列です（例：["PT", "GB"]）。ローカルの国データベース（セキュリティ > 国データベース、または MW_GEOIP_DB）のみを使用し、アドレスが位置情報サービスへ送信されることはありません。
admin-settings-key-security-signin-geo-unknown = 対応付けのない送信元アドレスのポリシー
admin-settings-help-security-signin-geo-unknown = プライベートアドレスを含め、読み取り可能なローカルデータベースにないアドレスを許可または拒否します。不明なアドレスを許可していても、データベースがない、または読み取れない場合は、有効なポリシーの対象のサインインは常に拒否されます。
admin-settings-key-contacts-avatar-mode = 連絡先の写真と生成されたアバター
admin-settings-help-contacts-avatar-mode = イニシャルをローカルで生成する、アバターを非表示にする、または各アカウント自身のアドレス帳に埋め込まれた写真を使用します。リモートの写真 URL や外部のアバターサービスには接続しません。ユーザーとドメインのポリシーは管理者が固定できます。
admin-settings-cat-contacts = 連絡先の写真
admin-settings-key-reader-action-style = メッセージ操作のリボン
admin-settings-help-reader-action-style = 既定はコンパクトなアイコンです。ラベルを選ぶと、テキスト付きのアイコンになります。どちらも操作名とテーマに沿ったツールチップを保持します。グローバル、ドメイン、ユーザーのポリシーによるロックに対応しています。
admin-settings-key-nextcloud-sync-interval-secs = Nextcloud の自動更新間隔
admin-settings-help-nextcloud-sync-interval-secs = 接続済みのネイティブエンジンのアカウントについて、バックグラウンドでコレクションを更新する間隔（秒）です。0 は手動のみで、正の値は最小 60 秒です。失敗した場合は間隔を空けて再試行します。バックグラウンドでのサインインは行われません。
admin-settings-key-mail-folder-role-inbox = 受信トレイのフォルダー割り当て
admin-settings-help-mail-folder-role-inbox = Null は自動検出を使用します。共有の既定値には正確なパスのオブジェクトを、1 つのアカウントには所有するメールボックス ID のオブジェクトを使用します。見つからない、または競合する対象は、この役割を無効にします。管理者のロックが適用されます。
admin-settings-key-mail-folder-role-drafts = 下書きのフォルダー割り当て
admin-settings-help-mail-folder-role-drafts = Null は自動検出を使用します。共有の既定値には正確なパスのオブジェクトを、1 つのアカウントには所有するメールボックス ID のオブジェクトを使用します。見つからない、または競合する対象は、この役割を無効にします。管理者のロックが適用されます。
admin-settings-key-mail-folder-role-sent = 送信済みのフォルダー割り当て
admin-settings-help-mail-folder-role-sent = Null は自動検出を使用します。共有の既定値には正確なパスのオブジェクトを、1 つのアカウントには所有するメールボックス ID のオブジェクトを使用します。見つからない、または競合する対象は、この役割を無効にします。管理者のロックが適用されます。
admin-settings-key-mail-folder-role-trash = ゴミ箱のフォルダー割り当て
admin-settings-help-mail-folder-role-trash = Null は自動検出を使用します。共有の既定値には正確なパスのオブジェクトを、1 つのアカウントには所有するメールボックス ID のオブジェクトを使用します。見つからない、または競合する対象は、この役割を無効にします。管理者のロックが適用されます。
admin-settings-key-mail-folder-role-junk = 迷惑メールのフォルダー割り当て
admin-settings-help-mail-folder-role-junk = Null は自動検出を使用します。共有の既定値には正確なパスのオブジェクトを、1 つのアカウントには所有するメールボックス ID のオブジェクトを使用します。見つからない、または競合する対象は、この役割を無効にします。管理者のロックが適用されます。
admin-settings-key-mail-folder-role-archive = アーカイブのフォルダー割り当て
admin-settings-help-mail-folder-role-archive = Null は自動検出を使用します。共有の既定値には正確なパスのオブジェクトを、1 つのアカウントには所有するメールボックス ID のオブジェクトを使用します。見つからない、または競合する対象は、この役割を無効にします。管理者のロックが適用されます。
admin-settings-key-mail-request-delivery-receipt = 既定で配信通知を要求
admin-settings-help-mail-request-delivery-receipt = アカウントが SMTP DSN に対応している場合、新しいメッセージで配信通知を選択した状態にします。ロックはこの保存された既定値を制御し、リクエスト自体を許可するかどうかは receipts.dsnAllowed が別に管理します。配信は、読まれたことの証明にはなりません。
admin-settings-key-security-signin-geo-enforce-sessions = 既存のセッションにも国ポリシーを適用
admin-settings-help-security-signin-geo-enforce-sessions = 既定ではオフです。設定したサインインの国の範囲を、認証済みのリクエスト、ネイティブの bearer アクセス、署名付きの閲覧コンテンツに適用します。ブロックされたクライアントもサインアウトはできます。すでに開いているストリームは再接続時に確認され、既存の接続が終了することはありません。
admin-settings-key-security-protection-rate-mode = リクエスト予算の集計
admin-settings-help-security-protection-rate-mode = 既定ではオフです。observe は拒否せずに予算の超過を記録し、enforce は一時的な 429 を返します。既存のプロトコルの同時実行数の上限は、これとは別に維持されます。
admin-settings-key-security-protection-adaptive-mode = 適応型リクエスト保護
admin-settings-help-security-protection-adaptive-mode = 既定ではオフです。observe は説明可能な型付きの異常と提案される対処を記録し、enforce は期間限定の一時的なスロットルまたはブロックを適用します。これは決定論的なカウンターのポリシーであり、AI による分類器ではありません。
admin-settings-key-security-protection-client-requests = 期間ごとのクライアントのリクエスト数
admin-settings-help-security-protection-client-requests = サインインしていない 1 つのアドレスからの期間ごとのリクエスト数と、すべてのサインインリクエストです。有効なメールボックスまたは管理者のセッションを持つリクエストには、代わりにそれ自身のより大きな許容量が使われます。0 でこの予算はオフになります。
admin-settings-key-security-protection-account-requests = 期間ごとのアカウントのリクエスト数
admin-settings-help-security-protection-account-requests = サインイン中の 1 つのメールボックスについて、そのセッション、デバイス、アドレスをまたいだ期間ごとのリクエスト数です。サインイン中の利用に余裕を持たせるため、匿名の許容量より高く保ちます。0 でこの予算はオフになります。
admin-settings-key-rate-limit-diagnostics-burst = 同時に実行できる診断
admin-settings-help-rate-limit-diagnostics-burst = 管理者セッションが続けて開始できる診断の実行とバックエンドのチェックの数です。すでに実行中のチェックを開始した場合は、再カウントせずにそのチェックが返されます。
admin-settings-key-rate-limit-diagnostics-per-minute = 1 分あたりの診断実行数
admin-settings-help-rate-limit-diagnostics-per-minute = 管理者セッションが 1 分ごとに回復する診断の実行とバックエンドのチェックの数です。
admin-settings-key-security-protection-admin-requests = 期間ごとの管理者のリクエスト数
admin-settings-help-security-protection-admin-requests = サインイン中の 1 人の管理者の期間ごとのリクエスト数です。匿名の許容量より高く保ちます。0 でこの予算はオフになります。
admin-settings-key-security-protection-rate-window-secs = リクエスト予算の期間（秒）
admin-settings-help-security-protection-rate-window-secs = 固定のリクエスト予算の間隔です。容量による負荷の遮断や通常の未検出の応答は、適応型の根拠にはなりません。
admin-settings-key-security-protection-adaptive-clients = 適応型保護でクライアントを監視
admin-settings-help-security-protection-adaptive-clients = 適応モードが有効なときに、この対象の区分を有効にします。アカウントと管理者は、本人確認の後にのみカウントされます。
admin-settings-key-security-protection-adaptive-accounts = 適応型保護でアカウントを監視
admin-settings-help-security-protection-adaptive-accounts = 適応モードが有効なときに、この対象の区分を有効にします。アカウントと管理者は、本人確認の後にのみカウントされます。
admin-settings-key-security-protection-adaptive-admins = 適応型保護で管理者を監視
admin-settings-help-security-protection-adaptive-admins = 適応モードが有効なときに、この対象の区分を有効にします。アカウントと管理者は、本人確認の後にのみカウントされます。
admin-settings-key-security-protection-short-window-secs = 短期の異常期間（秒）
admin-settings-help-security-protection-short-window-secs = スライディングの観測期間です。長い期間は数時間から 30 日までに対応し、バケットの丸めにより最大で期間の 64 分の 1 が余分に残ることがあります。
admin-settings-key-security-protection-short-minimum-samples = 短期の比率の最小サンプル数
admin-settings-help-security-protection-short-minimum-samples = 比率のトリガーが適用されるまでに必要な、観測された正常なリクエストと異常なリクエストの合計の最小数です。異常の絶対しきい値は、この最小数とは独立しています。
admin-settings-key-security-protection-short-absolute-anomalies = 短期の異常の絶対予算
admin-settings-help-security-protection-short-absolute-anomalies = 正常なリクエストで比率が薄まる場合でも、選択された異常がこの数まで蓄積するとトリガーされます。0 でこのトリガーは無効になります。
admin-settings-key-security-protection-short-ratio-percent = 短期の異常の比率（パーセント）
admin-settings-help-security-protection-short-ratio-percent = 最小サンプル数に達した後、選択された異常がこの割合に達するとトリガーされます。絶対値か比率のどちらかのトリガーで十分です。0 でこのトリガーは無効になります。
admin-settings-key-security-protection-long-window-secs = 長期の異常期間（秒）
admin-settings-help-security-protection-long-window-secs = スライディングの観測期間です。長い期間は数時間から 30 日までに対応し、バケットの丸めにより最大で期間の 64 分の 1 が余分に残ることがあります。
admin-settings-key-security-protection-long-minimum-samples = 長期の比率の最小サンプル数
admin-settings-help-security-protection-long-minimum-samples = 比率のトリガーが適用されるまでに必要な、観測された正常なリクエストと異常なリクエストの合計の最小数です。異常の絶対しきい値は、この最小数とは独立しています。
admin-settings-key-security-protection-long-absolute-anomalies = 長期の異常の絶対予算
admin-settings-help-security-protection-long-absolute-anomalies = 正常なリクエストで比率が薄まる場合でも、選択された異常がこの数まで蓄積するとトリガーされます。0 でこのトリガーは無効になります。
admin-settings-key-security-protection-long-ratio-percent = 長期の異常の比率（パーセント）
admin-settings-help-security-protection-long-ratio-percent = 最小サンプル数に達した後、選択された異常がこの割合に達するとトリガーされます。絶対値か比率のどちらかのトリガーで十分です。0 でこのトリガーは無効になります。
admin-settings-key-security-protection-normal-cap-per-bucket = バケットごとの正常ベースラインの上限
admin-settings-help-security-protection-normal-cap-per-bucket = スライディング期間の各バケットで学習される正常ベースラインに上限を設けます。独立した異常の絶対予算により、有効なリクエストを際限なく水増しして不正利用を隠すことを防ぎます。
admin-settings-key-security-protection-signals = 適応型の根拠シグナル
admin-settings-help-security-protection-signals = 信頼できる観測として、不正な形式、過大なサイズ、認証情報の拒否、権限の拒否、本文の期限超過、またはオプトインの管理ルートの不一致から選択します。管理ルートの探索には、エンティティ未検出の応答と静的アセットは含まれません。一般的なステータスコード、容量による拒否、User-Agent 文字列は、不正利用の根拠にはなりません。
admin-settings-key-security-protection-throttle-secs = 最初の一時的なスロットル（秒）
admin-settings-help-security-protection-throttle-secs = 最初にしきい値を超えると、固定のスロットルの期限が始まります。その期限内のリクエストで期限が延びることはありません。
admin-settings-key-security-protection-block-secs = 引き上げた一時的なブロック（秒）
admin-settings-help-security-protection-block-secs = 回復前に再びしきい値を超えると、この期間限定のブロックに引き上げられる場合があります。永続的なアカウントのロックは作成されません。
admin-settings-key-security-protection-decay-secs = 静穏による回復の間隔（秒）
admin-settings-help-security-protection-decay-secs = この間隔の間に異常がなく、現在の対処の有効期限が切れると、引き上げは最初の段階に戻ります。スライディングの根拠は、設定された期間に従って失効します。
admin-settings-key-security-protection-exempt-clients = 除外するクライアントのアドレスとネットワーク
admin-settings-help-security-protection-exempt-clients = 正確な IP アドレスまたは CIDR ネットワークで、クライアントの区分のみを除外します。検証済みのアカウントの上限は引き続き適用されます。共有 NAT ネットワークでは慎重に使用してください。転送されたアドレスは、設定されたプロキシ経由の場合にのみ信頼されます。
admin-settings-key-security-protection-exempt-accounts = 除外する検証済みアカウント ID
admin-settings-help-security-protection-exempt-accounts = サーバーが検証した正確なメールボックスのアカウント ID で、そのアカウントの区分のみを除外し、送信元クライアントの区分は除外しません。メールアドレスと、呼び出し側が指定したアカウントのパラメーターは、本人確認の根拠にはなりません。
admin-settings-key-security-protection-exempt-admins = 除外する管理者 ID
admin-settings-help-security-protection-exempt-admins = 認証済みの管理者の正確な ID で、その管理者の区分のみを除外します。既存のサインイン失敗によるロックと、厳格な管理者セッションの期限は変わりません。
admin-settings-key-security-protection-storage-failure = 適用時のストレージ障害のポリシー
admin-settings-help-security-protection-storage-failure = closed は、有効な適用予算を確認できない場合に保護対象の処理を拒否します。open は、運用上の限定的な警告を出したうえで一時的に許可します。観測のみの障害では、適用の対処は行われません。
admin-settings-key-security-admin-probe-enabled = 管理画面の探索への保護
admin-settings-help-security-admin-probe-enabled = 管理コンソールを探すリクエストをクライアントネットワーク単位（IPv4 アドレス、IPv6 /64）でカウントし、続ける送信元を遅くし、その後ブロックします。入力ミスや古いブックマークは猶予の範囲に収まります。セキュリティ > 適応型保護で変更します。
admin-settings-key-security-admin-probe-grace-allowance = 猶予の許容量（期間あたりの不一致の数）
admin-settings-help-security-admin-probe-grace-allowance = 期間ごとに無視される不一致の数です。スコアは、期間あたりこの点数のペースで継続的に減っていきます。
admin-settings-key-security-admin-probe-window-secs = 猶予期間（秒）
admin-settings-help-security-admin-probe-window-secs = 猶予の許容量が適用される期間です。
admin-settings-key-security-admin-probe-block-after = ブロックまでの猶予を超えた不一致の数
admin-settings-help-security-admin-probe-block-after = ブロックを開始する、猶予の許容量を超えた不一致の数です。その間の不一致には、タールピットの遅延の後に応答します。
admin-settings-key-security-admin-probe-tarpit-delay-ms = タールピットの遅延（ミリ秒）
admin-settings-help-security-admin-probe-tarpit-delay-ms = 猶予の許容量を超えてカウントされた不一致ごとに追加される遅延です。0 でタールピットはオフになります。その他のリクエストが遅延されることはありません。
admin-settings-key-security-admin-probe-block-secs = 最初のブロック（秒）
admin-settings-help-security-admin-probe-block-secs = 最初のブロックの長さです。ブロックされた送信元には、どのパスにも同じ応答が返されます。
admin-settings-key-security-admin-probe-max-block-secs = 最長のブロック（秒）
admin-settings-help-security-admin-probe-max-block-secs = 繰り返しのブロックでは、前回の長さが 2 倍になり、この上限までとなります。
admin-settings-key-security-admin-probe-repeat-window-secs = 繰り返しの期間（秒）
admin-settings-help-security-admin-probe-repeat-window-secs = 前回のブロックが終了してからこの期間内に始まったブロックは、繰り返しとして扱われます。
admin-settings-key-security-admin-probe-block-status = ブロック時の応答ステータス
admin-settings-help-security-admin-probe-block-status = ブロックされた送信元に Retry-After とともに送る HTTP ステータスです。429 または 403。
admin-settings-key-security-admin-probe-signals = カウントする不一致
admin-settings-help-security-admin-probe-signals = probe-path、slug-guess（コンソールのパスの深さでの 404）、admin-signin-failed、admin-unauthenticated。
admin-settings-key-security-admin-probe-probe-paths = 探索パス
admin-settings-help-security-admin-probe-probe-paths = /wp-admin や /.env など、スキャナーが要求するパスです。これらをパス全体のセグメントとして含むパスは不一致としてカウントされます。コンソール自身のパスはカウントされません。
admin-settings-key-security-admin-probe-allowlist = ブロックしないネットワーク
admin-settings-help-security-admin-probe-allowlist = カウントもブロックもされない IP アドレスまたは CIDR ネットワーク（オフィス、VPN）です。
admin-settings-key-security-admin-probe-audit-ip = 監査ログの送信元アドレス
admin-settings-help-security-admin-probe-audit-ip = masked は IPv4 /24 と IPv6 /48 を表示し、full はアドレス全体を表示します。fail2ban の行には、常に完全なアドレスが含まれます。
admin-settings-key-security-admin-probe-fail2ban-log = ブロック時に fail2ban のログ行を出力
admin-settings-help-security-admin-probe-fail2ban-log = 送信元がブロックされたときに、rhost=<address> を含む mailwoman[admin-probe] の行を 1 行書き込みます。
admin-settings-key-appearance-custom-css-enabled = 個人用の外観 CSS を許可
admin-settings-help-appearance-custom-css-enabled = 所有者のワークスペースで、制限付きの視覚スタイルを許可します。既定ではオフです。メールボックスの外観ロックが有効な場合、これらのスタイルは一時停止されます。
admin-settings-key-appearance-custom-js-enabled = 個人用の外観スクリプトを許可
admin-settings-help-appearance-custom-js-enabled = 外観の変更のみを要求できる、分離されたスクリプトを許可します。既定ではオフです。スクリプトは、メール、アカウントデータ、アプリケーションの DOM にアクセスできません。
admin-settings-key-mail-modifier-selection = メッセージの修飾キーによる選択
admin-settings-help-mail-modifier-selection = Ctrl、Command、Shift による範囲選択を許可します。この値をロックすると強制されます。表示される選択コントロールは引き続き使用できます。
admin-settings-key-mail-drag-to-folder = メッセージをフォルダーへドラッグ
admin-settings-help-mail-drag-to-folder = アカウントに紐づく、実際のフォルダーへのメッセージのドラッグを許可します。メールボックスへの書き込み権限を付与するものではなく、サーバーは引き続き各移動を承認します。
admin-settings-key-admin-allow-last-superadmin-loss = 最後のスーパー管理者の喪失を許可
admin-settings-help-admin-allow-last-superadmin-loss = 既定ではオフです。有効にすると、以降の変更で最後のスーパー管理者のアクセスを削除できるようになります。完全なスーパー管理者が、再認証のうえでその結果を確認する必要があります。
admin-settings-last-superadmin-title = すべてのスーパー管理者のアクセスの喪失を許可しますか？
admin-settings-last-superadmin-body = 以降のアカウントの変更により、このデプロイを管理できる人がいなくなる可能性があります。復旧にはローカルでのデプロイの管理が必要です。この例外を有効にするには、ALLOW LAST SUPERADMIN LOSS と入力してください。
admin-settings-last-superadmin-confirm = アクセスの喪失を許可
admin-settings-key-mail-folder-counts = フォルダーの件数
admin-settings-help-mail-folder-counts = フォルダーの横に、非表示、未読数、未読メールがあるときだけ未読数、合計数、または両方のどれを表示するかを選びます。グローバル、ドメイン、ユーザーの既定値を設定でき、必要に応じてその選択をロックできます。アカウントのリンクされたすべてのメールボックスに適用されます。
admin-settings-key-mail-folder-count-scope = 件数を表示するフォルダー
admin-settings-help-mail-folder-count-scope = すべてのフォルダー、または主要なフォルダーのみ（受信トレイ、下書き、送信済み、アーカイブ、迷惑メール、ゴミ箱、送信トレイなどの特別な用途のフォルダー）です。主要なフォルダーのみの場合、カスタムフォルダーとそのサブフォルダーには件数が表示されません。アカウントのリンクされたすべてのメールボックスに適用されます。
admin-settings-key-mail-compact-folder-badges = 副次的なフォルダーのコンパクトなバッジ
admin-settings-help-mail-compact-folder-badges = 受信トレイ以外で、より小さいバッジを表示します。この設定がロックされていない限り、ユーザーは自分でサイズを選べます。
admin-settings-key-mail-focused-inbox = 重要な受信トレイ
admin-settings-help-mail-focused-inbox = 受信トレイを「重要」と「その他」のタブに分けます。ニュースレターや自動送信者は「その他」に入ります。既定ではオフです。この設定がロックされていない限り、ユーザーは設定 > フォルダーとサイドバーでオンまたはオフにできます。
admin-settings-key-mail-account-unread-badge = メールボックス名の横の未読数
admin-settings-help-mail-account-unread-badge = サイドバーで、サインイン中のアドレスまたはニックネームの横に受信トレイの未読数を表示します。既定ではオフです。メールボックスメニューにある他のメールボックスの件数は常に表示されます。アカウントのリンクされたすべてのメールボックスに適用されます。
admin-settings-key-privacy-remote-image-downloads = リモート画像の手動ダウンロード
admin-settings-key-privacy-remote-images-toast = 画像読み込みの通知
admin-settings-help-privacy-remote-images-toast = メッセージのリモート画像がプライバシープロキシ経由で読み込まれている間、閲覧画面に通知を 1 つ表示し、読み込みが完了したとき、または読み込めなかった数を知らせます。既定ではオンです。変わるのは閲覧画面の表示だけで、どの画像を読み込むかは引き続きリモートコンテンツのポリシーで決まります。
admin-settings-help-privacy-remote-image-downloads = 保護されたプライバシープロキシを通じて、所有するメッセージが参照する画像を明示的にダウンロードすることを許可します。既定ではオフです。外部 URL を直接開くことはありません。
admin-settings-key-mail-resume-tabs = ワークスペースのタブを再開
admin-settings-help-mail-resume-tabs = 再読み込み後に、アカウントが所有するタブの参照を復元します。無効にすると、下書きを削除せずにローカルのワークスペースの記録が消去されます。
admin-settings-key-metrics-default-range-seconds = 既定のリクエスト履歴の範囲
admin-settings-help-metrics-default-range-seconds = 履歴のリクエストで範囲が指定されなかった場合に表示する秒数です。保持期間と最大範囲によって制限されます。
admin-settings-key-metrics-max-range-seconds = リクエスト履歴の最大範囲
admin-settings-help-metrics-max-range-seconds = 1 回の履歴クエリの最大秒数です。保持期間は追加の制限として引き続き適用されます。
admin-settings-key-contacts-favorites-first = 候補でお気に入りを先頭に
admin-settings-help-contacts-favorites-first = 宛先の候補で、お気に入りの連絡先を他の一致より前に表示します。変わるのは順序のみで、非表示になる候補はありません。ユーザーとドメインの上書きはロックできます。
admin-settings-key-contacts-sender-icons = 組み込みの送信者アイコン
admin-settings-help-contacts-sender-icons = 認識された送信者のドメインにはローカルに組み込まれたアイコンを表示し、それ以外には生成されたイニシャルを表示します。連絡先の写真が優先されます。アイコンは送信者の身元を確認するものではありません。外部のロゴサービスには接続しません。ユーザーとドメインの上書きはロックできます。
admin-settings-key-mail-brand-avatars = 検証済みの送信者の横にブランドロゴを表示
admin-settings-help-mail-brand-avatars = メッセージがそのブランド自身のドメインについて DKIM と整合した DMARC のチェックに合格した場合に限り、閲覧画面の送信者の横に、ローカルに組み込まれたブランドロゴを表示します。チェックに失敗した、または行われていないメッセージには通常のアバターが残ります。送信者やロゴサービスからロゴが取得されることはありません。
admin-settings-key-mail-draft-save-mode = アカウントの下書き保存モード
admin-settings-help-mail-draft-save-mode = 入力が止まった後、定期的に、または手動のときだけ保存します。保存には、対応するアップストリームの下書きフォルダーが必要です。結果が不確かな書き込みが自動的に再試行されることはありません。
admin-settings-key-mail-draft-debounce-ms = 下書きの入力の一時停止（ミリ秒）
admin-settings-help-mail-draft-debounce-ms = リアルタイムモードで、編集後に保存するまで 500 から 10000 ミリ秒待ちます。
admin-settings-key-mail-draft-interval-secs = 下書きの保存間隔（秒）
admin-settings-help-mail-draft-interval-secs = 定期モードで、編集された下書きを 5 から 3600 秒ごとに保存します。変更のない下書きは再び書き込まれません。
admin-settings-key-login-remember-signins = 成功したサインインのアドレスを記憶
admin-settings-help-login-remember-signins = サインインのドロップダウン用に、正常に使われたユーザー名またはメールアドレスを、このデバイスに最大 10 件まで保持します。パスワードやセッションは保存しません。無効にすると、デバイスが次にサインインページを読み込むか、サインインを完了したときに、このサーバーの保存済みアドレスが削除されます。ブラウザー自身の自動入力は制御しません。
admin-settings-key-login-show-remembered-hint = 保存済みアドレスの説明を表示
admin-settings-help-login-show-remembered-hint = 以前のサインインのドロップダウンに、説明のテキストを表示します。既定では非表示です。成功したサインインの記憶が有効な場合にのみ適用されます。
admin-settings-key-mail-show-from = 作成画面の差出人欄を表示
admin-settings-help-mail-show-from = 新しい作成画面で、差出人を既定で表示します。ユーザーは 1 通のメッセージについて表示できますが、送信できるのは許可されたアドレスのみです。
admin-settings-key-accounts-linking-enabled = リンクされたメールボックスアカウントを許可
admin-settings-help-accounts-linking-enabled = 既定では無効です。グローバルに無効にすると、リンクされたすべてのアクセスがブロックされます。両方のアカウントが、ドメインとアカウントのポリシーも満たす必要があります。リンクには、子アカウントの実際の認証と、最新のプライマリアカウントの確認が必要です。
admin-settings-key-accounts-use-linked = リンクされたメールボックスを使用
admin-settings-help-accounts-use-linked = リンクされたアクセスについての、ユーザーによるオプトアウトです。管理者による無効化は上書きできません。アクセスが無効の間も、メールボックスのデータは保持されます。
admin-settings-key-accounts-max-linked = リンクされたメールボックスの最大数
admin-settings-help-accounts-max-linked = 1 つのプライマリ ID あたりの、検証済みの子メールボックスの最大数で、1 から 16 までです。既定は 5 です。
admin-settings-key-accounts-store-linked-credentials = リンクされたメールボックスのアクセスを維持
admin-settings-help-accounts-store-linked-credentials = 各リンクされたメールボックスのパスワードをサーバー上で保護して保持し、セッションや再起動をまたいで、アクセスが自動的に更新されるようにします。ゼロアクセスのアカウントや、2 段階認証のあるメールボックスでは保持されません。オフにすると、保存されたパスワードが消去され、リンクは再度サインインすることで更新されます。既定ではオンです。
admin-settings-key-privacy-strip-mailer-headers = 送信メールのメーラーヘッダーを削除
admin-settings-help-privacy-strip-mailer-headers = ネイティブの SMTP および EWS のメールから、最上位の X-Mailer、User-Agent、X-MimeOLE を削除します。添付されたメッセージとレポートのペイロードはそのまま残ります。アップストリームの JMAP プロキシ経由の送信は対象外です。
admin-settings-key-images-trusted-hosts = 信頼する画像ホスト
admin-settings-help-images-trusted-hosts = リモートコンテンツが「確認」のとき、メッセージごとの許可なしでリモート画像を読み込む画像ホストとそのサブドメインを、最大 100 件まで登録できます。ブロックモードを上書きすることはなく、ブロックされたホストはブロックされたままです。
admin-settings-key-images-blocked-hosts = ブロックする画像ホスト
admin-settings-help-images-blocked-hosts = どのモードでもリモート画像を読み込まない画像ホストとそのサブドメインを、最大 100 件まで登録できます。信頼するホストや、あらゆる許可より優先されます。
admin-settings-key-links-trusted-domains = 信頼するリンクのドメイン
admin-settings-help-links-trusted-domains = 「リンクを確認」から、リンク先の確認の手順なしでリンクが開く正確なドメインを、最大 100 件まで登録できます。ワイルドカード、スキーム、ポート、IP アドレスは使用できません。HTTP のみのリンクは常に確認されます。ユーザーが一覧を変更できないようにするには、この設定をロックします。
admin-settings-key-links-trusted-subdomains = 信頼するリンクのドメインのサブドメインも信頼
admin-settings-help-links-trusted-subdomains = オフ：正確なドメインのみを信頼します。オン：.<domain> で終わる名前も信頼します。照合はラベル全体で行われるため、example.com が evilexample.com を信頼することはありません。
admin-settings-key-cap-compose-rich-text = リッチテキストでの作成
admin-settings-help-cap-compose-rich-text = ユーザーが書式付き（HTML）のメッセージを書けるかどうかです。オフの場合は、プレーンテキストのみを送信します。
admin-settings-key-cap-compose-attachments = 新規メッセージの添付ファイル
admin-settings-help-cap-compose-attachments = ユーザーが自分で書くメッセージにファイルを添付できるかどうかです。本文中のインライン画像は含まれません。
admin-settings-key-cap-compose-share-links = 大容量ファイルの共有リンク
admin-settings-help-cap-compose-share-links = ユーザーが、大きなファイルを添付ファイルの代わりに Nextcloud の共有リンクとして送信できるかどうかです。
admin-settings-key-cap-compose-scheduled-send = 送信予約
admin-settings-help-cap-compose-scheduled-send = ユーザーがメッセージを後で送信するよう予約できるかどうかです。
admin-settings-key-cap-compose-read-receipts = 開封確認の要求
admin-settings-help-cap-compose-read-receipts = ユーザーが受信者に開封確認を求められるかどうかです。
admin-settings-key-cap-signatures = 署名
admin-settings-help-cap-signatures = ユーザーが自分の署名を作成および編集できるかどうかです。
admin-settings-key-cap-identities = 差出人として送信する ID
admin-settings-help-cap-identities = ユーザーが、送信元として使う ID を追加および編集できるかどうかです。
admin-settings-key-cap-keys = OpenPGP と S/MIME のキー
admin-settings-help-cap-keys = ユーザーがキーモジュールを利用でき、メッセージへの署名と暗号化を行えるかどうかです。
admin-settings-key-cap-links-trusted = 信頼するリンクのドメインの編集
admin-settings-help-cap-links-trusted = ユーザーが、信頼するリンクのドメインの自分用の一覧を変更できるかどうかです。管理者の一覧は引き続き適用されます。
admin-settings-key-cap-zoom = 閲覧画面のズーム
admin-settings-help-cap-zoom = ユーザーがメッセージの閲覧画面をズームできるかどうかです。
admin-settings-key-cap-mail-focused-inbox = 重要な受信トレイ
admin-settings-help-cap-mail-focused-inbox = ユーザーが、受信トレイの「重要」と「その他」のタブをオンにできるかどうかです。
admin-settings-key-cap-views-events = イベントビュー
admin-settings-help-cap-views-events = ユーザーが、カレンダーメールのイベントビューを利用できるかどうかです。
admin-settings-key-cap-views-attachments = 添付ファイルビュー
admin-settings-help-cap-views-attachments = ユーザーが添付ファイルビューを利用できるかどうかです。メッセージは検索から引き続き見つけられます。
admin-settings-key-cap-search-operators = 検索演算子
admin-settings-help-cap-search-operators = ユーザーが from:、subject:、has:attachment などの演算子で検索できるかどうかです。
admin-settings-key-cap-saved-searches = 保存した検索
admin-settings-help-cap-saved-searches = ユーザーが検索を保存できるかどうかです。
admin-settings-key-cap-vacation = 不在時の自動返信
admin-settings-help-cap-vacation = ユーザーが不在時の自動返信を設定できるかどうかです。
admin-settings-key-cap-labels = ラベル
admin-settings-help-cap-labels = ユーザーがメッセージにラベルを付けられるかどうかです。
admin-settings-key-cap-snooze = スヌーズ
admin-settings-help-cap-snooze = ユーザーがメッセージをスヌーズできるかどうかです。スヌーズの解除は引き続き可能です。
admin-settings-key-cap-follow-up = フォローアップのリマインダー
admin-settings-help-cap-follow-up = ユーザーがメッセージにフォローアップのリマインダーを設定できるかどうかです。解除は引き続き可能です。
admin-settings-key-cap-masked = マスクされたアドレス
admin-settings-help-cap-masked = ユーザーがマスクされたアドレスを作成および管理できるかどうかです。
admin-settings-key-cap-directory = ディレクトリ検索
admin-settings-help-cap-directory = ユーザーが組織のディレクトリを検索できるかどうかです。
admin-settings-key-cap-mcp = MCP アクセス
admin-settings-help-cap-mcp = ユーザーのキーとトークンが MCP エンドポイントを使用できるかどうかです。
admin-settings-key-cap-appearance-custom = 外観のカスタマイズ
admin-settings-help-cap-appearance-custom = ユーザーが、テーマ、アクセント、カスタムとバッジの色、密度、フォントを変更できるかどうかです。オフの場合は、管理者が設定した外観が維持されます。
admin-settings-key-cap-appearance-custom-code = カスタム CSS とスクリプト
admin-settings-help-cap-appearance-custom-code = ユーザーが自分の CSS と外観のスクリプトを保存できるかどうかです。外観にあるデプロイのスイッチが、これより上位で引き続き適用されます。
admin-settings-key-cap-avatars = メールボックスのアバター
admin-settings-help-cap-avatars = ユーザーがメールボックスのアバターを変更できるかどうかです。
admin-settings-key-cap-shortcuts = カスタムのキーボードショートカット
admin-settings-help-cap-shortcuts = ユーザーがキーボードショートカットを変更できるかどうかです。既定のショートカットは引き続き動作します。
admin-settings-key-cap-analytics = プロダクト分析
admin-settings-help-cap-analytics = ユーザーに分析への同意を求め、利用イベントの送信を許可するかどうかです。
admin-settings-key-cap-apps-native = デスクトップアプリとモバイルアプリ
admin-settings-help-cap-apps-native = ユーザーがデスクトップアプリとモバイルアプリを使えるかどうかです。
admin-settings-key-access-protocols = メールアクセスのプロトコル
admin-settings-help-access-protocols = これらのユーザーがサインインに使えるメールサーバーのプロトコルです：imap、pop3、ews。JMAP サーバー経由のサインインは、そのサーバーに従います。
admin-settings-key-limit-linked-mailboxes = ユーザーごとのリンクされたメールボックス数
admin-settings-help-limit-linked-mailboxes = サーバーの最大数の範囲内で、ユーザーがリンクできるメールボックスの数です。0 の場合は、サーバーの最大数のみが適用されます。
admin-settings-key-login-show-account-help-notice = サインイン時にアカウントのヘルプのお知らせを表示
admin-settings-help-login-show-account-help-notice = サインインに失敗した後、短い説明を表示します。既定では非表示です。このお知らせから、アカウントが存在するか、または無効かどうかが分かることはありません。
admin-settings-key-appearance-leetspeak = Leetspeak のテキストスタイル
admin-settings-help-appearance-leetspeak = インターフェイスのラベルと読み取り専用のメッセージテキストを leetspeak で表示します。元のメッセージと編集可能なテキストは変わりません。この設定がロックされていない限り、ユーザーは通常のテキストを選べます。
admin-settings-key-mail-cache-enabled = セッションキャッシュを有効にする
admin-settings-help-mail-cache-enabled = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みで、メールが既読になることやリモート画像が読み込まれることはありません。
admin-settings-key-mail-cache-mailboxes = フォルダーのメッセージ一覧をキャッシュ
admin-settings-help-mail-cache-mailboxes = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みで、メールが既読になることやリモート画像が読み込まれることはありません。
admin-settings-key-mail-cache-messages = 開いたメッセージをキャッシュ
admin-settings-help-mail-cache-messages = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みで、メールが既読になることやリモート画像が読み込まれることはありません。
admin-settings-key-mail-cache-fresh-seconds = 鮮度の間隔（秒）
admin-settings-help-mail-cache-fresh-seconds = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みで、メールが既読になることやリモート画像が読み込まれることはありません。
admin-settings-key-mail-cache-max-age-seconds = キャッシュの最大保持時間（秒）
admin-settings-help-mail-cache-max-age-seconds = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みで、メールが既読になることやリモート画像が読み込まれることはありません。
admin-settings-key-mail-cache-max-entries = キャッシュするエントリーの最大数
admin-settings-help-mail-cache-max-entries = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みで、メールが既読になることやリモート画像が読み込まれることはありません。
admin-settings-key-mail-cache-max-bytes = メモリの予算（バイト）
admin-settings-help-mail-cache-max-bytes = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みで、メールが既読になることやリモート画像が読み込まれることはありません。
admin-settings-key-mail-cache-prefetch-enabled = 予測的な先読みを有効にする
admin-settings-help-mail-cache-prefetch-enabled = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みで、メールが既読になることやリモート画像が読み込まれることはありません。
admin-settings-key-mail-cache-hover-enabled = フォルダーにポインターを合わせたときに先読み
admin-settings-help-mail-cache-hover-enabled = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みで、メールが既読になることやリモート画像が読み込まれることはありません。
admin-settings-key-mail-cache-focus-enabled = フォルダーにキーボードフォーカスが移ったときに先読み
admin-settings-help-mail-cache-focus-enabled = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みで、メールが既読になることやリモート画像が読み込まれることはありません。
admin-settings-key-mail-cache-dwell-ms = ポインターとフォーカスの遅延（ミリ秒）
admin-settings-help-mail-cache-dwell-ms = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みで、メールが既読になることやリモート画像が読み込まれることはありません。
admin-settings-key-mail-cache-idle-enabled = アイドル中にフォルダーを先読み
admin-settings-help-mail-cache-idle-enabled = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みで、メールが既読になることやリモート画像が読み込まれることはありません。
admin-settings-key-mail-cache-idle-delay-ms = アイドルの遅延（ミリ秒）
admin-settings-help-mail-cache-idle-delay-ms = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みで、メールが既読になることやリモート画像が読み込まれることはありません。
admin-settings-key-mail-cache-max-concurrency = 同時に先読みできる最大数

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-cache-max-concurrency = 最近開いたフォルダーとメッセージをこのタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みでメッセージが既読になったり、リモート画像が読み込まれたりすることはありません。
admin-settings-key-mail-cache-max-queued = 先読みキューの最大数
admin-settings-help-mail-cache-max-queued = 最近開いたフォルダーとメッセージをこのタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みでメッセージが既読になったり、リモート画像が読み込まれたりすることはありません。
admin-settings-key-mail-cache-preload-messages = メッセージ本文の先読みを許可
admin-settings-help-mail-cache-preload-messages = 最近開いたフォルダーとメッセージをこのタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みでメッセージが既読になったり、リモート画像が読み込まれたりすることはありません。
admin-settings-key-mail-cache-excluded-folders = 先読みから除外するフォルダー
admin-settings-help-mail-cache-excluded-folders = 最近開いたフォルダーとメッセージをこのタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかサインアウトすると消去されます。先読みでメッセージが既読になったり、リモート画像が読み込まれたりすることはありません。
admin-settings-key-mail-cache-refresh-on-open = 開くときにキャッシュを更新
admin-settings-help-mail-cache-refresh-on-open = 開くときにキャッシュされた内容を更新します
admin-settings-key-mail-templates-enabled = 返信テンプレート
admin-settings-help-mail-templates-enabled = 作成画面で、個人用と共有のプレーンテキスト返信テンプレートを使えるようにします。この設定がロックされていない限り、ユーザーは使用をオフにできます。
admin-settings-key-mail-templates-allow-personal = 個人用テンプレートを許可
admin-settings-help-mail-templates-allow-personal = ユーザーが自分の返信テンプレートを作成できるようにします。共有テンプレートは管理者が別途管理します。
admin-settings-key-mail-templates-shared = 共有返信テンプレート
admin-settings-help-mail-templates-shared = このスコープで再利用できるプレーンテキストの返信を管理します。テンプレートが宛先を設定したりメッセージを送信したりすることはありません。
admin-settings-key-mail-templates-personal = 個人用返信テンプレート
admin-settings-help-mail-templates-personal = 個人用の再利用できるプレーンテキストの返信です。作成や編集には、返信テンプレートと個人用テンプレートの両方が有効である必要があります。
admin-settings-key-mail-compose-window = 作成ウィンドウ
admin-settings-help-mail-compose-window = 新しい下書きをワークスペースのタブまたは別ウィンドウで開きます。ポップアップがブロックされた場合はタブで開きます。
admin-settings-key-mail-reply-copy-to-original = 返信を元のフォルダーに残す
admin-settings-help-mail-reply-copy-to-original = 対応している場合、送信した返信のコピーを元のメッセージがあるフォルダーに保存します。
admin-settings-key-mail-compose-check-invalid-recipients = 送信前に無効な宛先を確認
admin-settings-help-mail-compose-check-invalid-recipients = 無効なアドレスにメッセージを送信するときの動作です。許可、警告、警告して選択を記憶、拒否から選びます。
admin-settings-key-mail-compose-check-missing-attachment = 送信前に添付忘れを確認
admin-settings-help-mail-compose-check-missing-attachment = 本文に添付ファイルへの言及があるのに添付がないときの動作です。許可、警告、警告して選択を記憶、拒否から選びます。
admin-settings-key-mail-compose-check-empty-subject = 送信前に件名の未入力を確認
admin-settings-help-mail-compose-check-empty-subject = メッセージに件名がないときの動作です。許可、警告、警告して選択を記憶、拒否から選びます。
admin-settings-key-mail-compose-auto-chip-recipients = 入力した宛先を自動で確定
admin-settings-help-mail-compose-auto-chip-recipients = 入力したアドレスの後にスペース、カンマ、セミコロンが続くと、宛先チップに変換します。
admin-settings-key-mail-compose-header-collapse = 作成画面のヘッダー
admin-settings-help-mail-compose-header-collapse = 作成画面の宛先、Cc、Bcc、件名の行の動作です。展開、1 行の概要に折りたたみ、または自動（本文の入力中は折りたたみ、項目にフォーカスがあるときは展開）から選びます。ロックされていない限り、ユーザーが自分で折りたたみや展開を切り替えられます。
admin-settings-key-mail-compose-recipient-separator = 宛先の区切り文字
admin-settings-help-mail-compose-recipient-separator = 宛先の確定後に挿入する区切り文字です。カンマまたはセミコロンから選びます。
admin-settings-key-mail-reply-from-mode = 返信と転送に使う ID
admin-settings-help-mail-reply-from-mode = 返信や転送の送信に使う ID です。元のメッセージの宛先になっていた ID、デフォルト、選択画面、現在選択中の ID から選びます。
admin-settings-key-mail-forward-mode = デフォルトの転送形式
admin-settings-help-mail-forward-mode = テキストをインラインで転送するか、元のメッセージを .eml ファイルとして添付して転送します。
admin-settings-key-mail-show-go-to-top = 「先頭へ移動」ボタンを表示
admin-settings-help-mail-show-go-to-top = メッセージ一覧に、先頭のメッセージまでスクロールして戻る「先頭へ移動」ボタンを表示します。デフォルトではオフです。
admin-settings-key-mail-show-create-folder-button = 「フォルダーを作成」ボタンを表示
admin-settings-help-mail-show-create-folder-button = サイドバーの「作成」の横に、アイコンのみの「フォルダーを作成」ボタンを表示します。デフォルトではオフです。
admin-settings-key-security-protection-user-agent-mode = User-Agent 監視モード
admin-settings-help-security-protection-user-agent-mode = デフォルトは強制です。最初のリクエストでのボット遮断が有効な場合、選択した既知のボットヘッダーを直ちに遮断します。監視は拒否せず判定のみを記録し、オフにするとこの観点を無効にします。ヘッダーの主張は検証済みの ID ではありません。
admin-settings-key-security-protection-user-agent-block-bots = 最初のリクエストで既知のボットを遮断
admin-settings-help-security-protection-user-agent-block-bots = デフォルトはオンです。強制モードでは、選択した検索、AI、自動化、カスタムのボットヘッダーが最初のリクエストで一時的な HTTP 429 の遮断を引き起こします。監視モードでは記録のみ行い、オフにすると以前のリクエスト枠に戻ります。ヘッダーがない場合や一般的でない場合、スコープ、例外の動作は従来どおりです。
admin-settings-key-security-protection-user-agent-categories = User-Agent のカテゴリ
admin-settings-help-security-protection-user-agent-categories = オプションの User-Agent 監視のこの部分を設定します。クライアントが提供する User-Agent の文字列は弱い手がかりであり、身元や不正の証拠ではありません。
admin-settings-key-security-protection-user-agent-weak-requests = 弱いシグナルのリクエスト閾値
admin-settings-help-security-protection-user-agent-weak-requests = オプションの User-Agent 監視のこの部分を設定します。クライアントが提供する User-Agent の文字列は弱い手がかりであり、身元や不正の証拠ではありません。
admin-settings-key-security-protection-user-agent-crawler-requests = クローラーのリクエスト閾値
admin-settings-help-security-protection-user-agent-crawler-requests = オプションの User-Agent 監視のこの部分を設定します。クライアントが提供する User-Agent の文字列は弱い手がかりであり、身元や不正の証拠ではありません。
admin-settings-key-security-protection-user-agent-window-secs = User-Agent 監視ウィンドウ（秒）
admin-settings-help-security-protection-user-agent-window-secs = オプションの User-Agent 監視のこの部分を設定します。クライアントが提供する User-Agent の文字列は弱い手がかりであり、身元や不正の証拠ではありません。
admin-settings-key-security-protection-user-agent-max-action = User-Agent に対する最大の対応
admin-settings-help-security-protection-user-agent-max-action = 弱いシグナルや繰り返しリクエストの証拠に対する最大のエスカレーションです。デフォルトはスロットルです。この上限は、選択した既知のボットヘッダーに対する最初のリクエストでの遮断には適用されません。
admin-settings-key-security-protection-user-agent-scope = User-Agent ポリシーのスコープ
admin-settings-help-security-protection-user-agent-scope = オプションの User-Agent 監視のこの部分を設定します。クライアントが提供する User-Agent の文字列は弱い手がかりであり、身元や不正の証拠ではありません。
admin-settings-key-security-protection-user-agent-adaptive = 適応型 User-Agent 監視
admin-settings-help-security-protection-user-agent-adaptive = オプションの User-Agent 監視のこの部分を設定します。クライアントが提供する User-Agent の文字列は弱い手がかりであり、身元や不正の証拠ではありません。
admin-settings-key-security-protection-user-agent-patterns = User-Agent パターン
admin-settings-help-security-protection-user-agent-patterns = オプションの User-Agent 監視のこの部分を設定します。クライアントが提供する User-Agent の文字列は弱い手がかりであり、身元や不正の証拠ではありません。
admin-settings-key-security-protection-user-agent-exceptions = User-Agent の例外
admin-settings-help-security-protection-user-agent-exceptions = オプションの User-Agent 監視のこの部分を設定します。クライアントが提供する User-Agent の文字列は弱い手がかりであり、身元や不正の証拠ではありません。
admin-settings-key-mail-reader-unsafe-html-mode = 安全でない可能性のあるメッセージ HTML
admin-settings-help-mail-reader-unsafe-html-mode = デフォルトの「確認」では、メッセージごとにサニタイズ済み HTML のボタンを表示します。「厳格」は安全なテキストのみを表示し、「サニタイズ」はクリーニング済みの HTML を自動で表示します。アクティブコンテンツと自動のリモートリソースは引き続きブロックされます。ロックするとこのポリシーが強制されます。
admin-settings-key-reader-attachments-density = 添付ファイルの密度
admin-settings-help-reader-attachments-density = デフォルトはコンパクトなファイルボタンで、ゆったりしたプレビューも選べます。メッセージの添付ファイルを展開したときの表示を制御します。
admin-settings-key-cap-compose-open-tracking = メッセージの開封トラッキングを許可
admin-settings-help-cap-compose-open-tracking = このアカウントがメッセージの開封を記録できるかどうかです。公開 URL の設定が必要です。無効なアカウントはトラッキングトークンを作成したり紐づけたりできません。
admin-settings-key-mail-open-tracking = デフォルトで開封をトラッキング
admin-settings-help-mail-open-tracking = 対象の書式付きメッセージに対する、メッセージごとのトラッキングのデフォルト設定です。ロックされていない限りユーザーが上書きできます。トラッキング機能を無効にした場合は常にそちらが優先されます。
admin-settings-key-cap-reading-original-html = 元の HTML のレンダリングを許可
admin-settings-help-cap-reading-original-html = ユーザーがメッセージごとに元の HTML 表示を選択できるかどうかです。デフォルトのサニタイズ済みレンダラーと分離フレームによるセキュリティは維持されます。
admin-settings-key-mail-reader-allow-original-html = 元の HTML 表示を提供
admin-settings-help-mail-reader-allow-original-html = 閲覧機能が許可している場合に、メッセージごとに元の HTML を使うオプションを表示します。デフォルトではオフで、デフォルトのメッセージレンダラーは変わりません。
admin-settings-key-privacy-tracker-override-allowed = メッセージごとのトラッカー許可を提供
admin-settings-help-privacy-tracker-override-allowed = メッセージのプライバシーバーに「トラッカーを許可」操作を表示します。デフォルトで利用できますが、明示的な許可がない限り有効にはなりません。ロックされたトラッカーブロックポリシーは引き続き上書きを禁止します。
admin-settings-key-privacy-max-concurrent-global = 画像取得のグローバル並列数
admin-settings-help-privacy-max-concurrent-global = このサーバープロセス全体で同時にアクティブな画像ネットワークリクエストの最大数です。範囲は 1~256、デフォルトは 128 です。メッセージごとやアカウントごとの制限も引き続き適用されます。
admin-settings-key-privacy-max-concurrent-pipeline = 画像処理パイプラインの同時実行数
admin-settings-help-privacy-max-concurrent-pipeline = 取得、デコード、キャッシュ公開を同時に行える画像の最大数です。範囲は 1~256、デフォルトは 128 です。ネットワークとデコーダーの上限は別に適用されます。
admin-settings-key-privacy-max-concurrent-decodes = 画像デコーダーの並列数
admin-settings-help-privacy-max-concurrent-decodes = 同時に実行する画像デコードの最大数です。範囲は 1~16、デフォルトは 10 です。値を大きくすると CPU とメモリの使用量が増えます。
admin-settings-key-privacy-max-queued-requests = 待機中の画像リクエスト数
admin-settings-help-privacy-max-queued-requests = 画像のネットワークまたは処理の空きを待つリクエストの最大数です。範囲は 16~16384、デフォルトは 2048 です。溢れた場合やキューの期限切れの場合は再試行できます。
admin-settings-key-privacy-rate-burst = 画像リクエストのバースト許容量
admin-settings-help-privacy-rate-burst = アカウントごとの取得トークンの初期値かつ最大値で、設定された画像リクエストレートで補充されます。範囲は 1~12000、デフォルトは 1024 です。キャッシュで成功した場合はトークンを消費しません。
admin-settings-key-privacy-image-rate-mode = 画像リクエストレートの調整方法
admin-settings-help-privacy-image-rate-mode = 自動では、レプリカの容量、キューの負荷、メッセージの画像数に応じて、下の下限と上限の範囲内でアカウントごとのバーストと補充を調整します。手動ではバースト許容量とリクエストレートの設定をそのまま使います。デフォルトは自動です。許容量を超えたリクエストは、取得キューの期限までトークンを待ちます。認証と許可が先に確認されます。
admin-settings-key-privacy-image-rate-burst-floor = 自動バースト許容量の下限
admin-settings-help-privacy-image-rate-burst-floor = 自動レートモードでのアカウントごとのバースト許容量の最小値です。範囲は 1~12000、デフォルトは 64 です。
admin-settings-key-privacy-image-rate-burst-ceiling = 自動バースト許容量の上限
admin-settings-help-privacy-image-rate-burst-ceiling = 自動レートモードでのアカウントごとのバースト許容量の最大値です。範囲は 1~12000、デフォルトは 2048 です。
admin-settings-key-privacy-image-rate-refill-floor = 自動補充レートの下限
admin-settings-help-privacy-image-rate-refill-floor = 自動レートモードでのアカウントごとの補充の最小値で、1 分あたりの取得トークン数です。範囲は 10~60000、デフォルトは 300 です。
admin-settings-key-privacy-image-rate-refill-ceiling = 自動補充レートの上限
admin-settings-help-privacy-image-rate-refill-ceiling = 自動レートモードでのアカウントごとの補充の最大値で、1 分あたりの取得トークン数です。範囲は 10~60000、デフォルトは 6000 です。
admin-settings-key-privacy-image-cache-mode = 画像キャッシュのサイズ調整方法
admin-settings-help-privacy-image-cache-mode = 自動では、デコーダーとパイプラインの予約分を除いた残りのメモリからキャッシュサイズを決め、ヒット率が高くメモリに余裕があるあいだは保持期間を延ばします。手動ではキャッシュサイズ、エントリ数、保持期間の設定をそのまま使います。デフォルトは自動です。
admin-settings-key-privacy-image-cache-bytes-floor = 自動キャッシュサイズの下限
admin-settings-help-privacy-image-cache-bytes-floor = 自動モードでの画像キャッシュの最小サイズ（バイト）です。範囲は 0~1073741824、デフォルトは 16777216 です。
admin-settings-key-privacy-image-cache-bytes-ceiling = 自動キャッシュサイズの上限
admin-settings-help-privacy-image-cache-bytes-ceiling = 自動モードでの画像キャッシュの最大サイズ（バイト）です。範囲は 0~1073741824、デフォルトは 268435456 です。
admin-settings-key-privacy-cache-entries = 画像キャッシュのエントリ上限
admin-settings-help-privacy-cache-entries = キャッシュする画像の最大数です。手動モードでは固定値、自動モードでは上限になります。範囲は 16~65536、デフォルトは 4096 です。
admin-settings-key-privacy-cache-entry-bytes = キャッシュする画像の最大サイズ
admin-settings-help-privacy-cache-entry-bytes = キャッシュに保持する、再エンコード後の画像 1 枚あたりの最大サイズ（バイト）です。これより大きい画像は配信されますが保持されません。範囲は 65536~16777216、デフォルトは 4194304 です。
admin-settings-key-privacy-image-cache-ttl-floor-secs = 自動キャッシュ保持期間の下限
admin-settings-help-privacy-image-cache-ttl-floor-secs = 自動モードでの最短のキャッシュ保持期間（秒）で、ヒット率が低いときやメモリが逼迫しているときに使われます。範囲は 0~86400、デフォルトは 300 です。
admin-settings-key-privacy-image-cache-ttl-ceiling-secs = 自動キャッシュ保持期間の上限
admin-settings-help-privacy-image-cache-ttl-ceiling-secs = 自動モードでの最長のキャッシュ保持期間（秒）で、ヒット率が高くメモリに余裕があるときに使われます。範囲は 0~86400、デフォルトは 21600 です。
admin-settings-key-privacy-cache-ttl-secs = 成功した画像のキャッシュ保持期間
admin-settings-help-privacy-cache-ttl-secs = デコード済みの画像をこのプロセスで再利用できる秒数です。範囲は 0~86400、デフォルトは 3600 です。0 にすると成功した画像の再利用を無効にします。認可は常に先に確認されます。
admin-settings-key-privacy-image-latency-tracking = 画像プロキシの速度トラッキング
admin-settings-help-privacy-image-latency-tracking = プロキシした各画像の所要時間（中央値、p95、p99、および待機・取得・デコードにかかった時間）を、固定の時間バケットごとの件数としてメモリに記録します。URL、ホスト、アカウントは保存されません。デフォルトではオンで、履歴は再起動でリセットされ、サーバープロセスごとに保持されます。オフにすると新しい記録を停止しますが、「速度」セクションには記録済みの内容が引き続き表示されます。
admin-settings-key-privacy-negative-cache-ttl-secs = 失敗した画像のキャッシュ保持期間
admin-settings-help-privacy-negative-cache-ttl-secs = 該当する上流の失敗を、繰り返しのリクエストを避けるために記憶しておく秒数です。範囲は 0~300、デフォルトは 10 です。0 にすると失敗の再利用を無効にします。明示的に再試行すると、該当する失敗が消去されます。
admin-settings-key-privacy-image-retry-attempts = 画像の自動再試行回数
admin-settings-help-privacy-image-retry-attempts = 表示中のメッセージを 1 回読み込む間に、一時的な画像の失敗を再試行する最大回数です。範囲は 0~8、デフォルトは 3 です。0 にすると自動再試行を無効にし、手動での再試行は残ります。
admin-settings-key-privacy-image-retry-delay-ms = 画像の自動再試行の間隔
admin-settings-help-privacy-image-retry-delay-ms = 現在のフレームが落ち着いてから、画像の自動再試行までのミリ秒数です。範囲は 500~60000、デフォルトは 5000 です。恒久的に拒否された場合は自動で再試行されません。
admin-settings-key-privacy-image-decode-timeout-ms = 画像デコードのタイムアウト
admin-settings-help-privacy-image-decode-timeout-ms = 取得後のサンドボックス内での画像デコード 1 回に許可される実時間（ミリ秒）です。範囲は 1000~60000、デフォルトは 10000 です。大きな画像では増やしてください。ネットワークのタイムアウトは別です。
admin-settings-key-privacy-image-concurrency-mode = 画像の同時実行プロファイル
admin-settings-help-privacy-image-concurrency-mode = 自動では、CPU の容量、コンテナーのメモリ、各メッセージで承認された画像数に合わせて調整します。「低リソース」「バランス」「高スループット」も、異なる安全上限で適応します。手動では数値の上限を直接使います。プロファイルを適用しても、変更されるのはこの設定のみです。
admin-settings-cat-image-proxy = 画像プロキシ
admin-settings-key-privacy-image-ram-max-mi-b = RAM 階層の最大サイズ
admin-settings-help-privacy-image-ram-max-mi-b = メモリ内の画像階層の上限（MiB）で、自動または手動のキャッシュサイズに加えて適用されます。0 にすると上限を設けません。小さく保ち、大半はディスク階層に任せてください。範囲は 0~1048576、デフォルトは 0 です。
admin-settings-key-privacy-image-disk-enabled = 画像ディスク階層
admin-settings-help-privacy-image-disk-enabled = 再エンコード済みの画像を、RAM 階層の背後にあるディスクに保持します。デフォルトではオフです。ディレクトリがない、満杯、または読み取り専用の場合、キャッシュは通知なく RAM のみで動作を続けます。保存されるのはジェイルの出力のみで、ハッシュ化されたファイル名が使われます。
admin-settings-key-privacy-image-disk-path = 画像ディスクのディレクトリ
admin-settings-help-privacy-image-disk-path = ディスク階層に使う絶対パスのディレクトリです。空の場合は、データベースファイルの隣にある image-cache フォルダーを使います。ルートファイルシステムが読み取り専用の場合は、書き込み可能なボリュームをマウントしてください。相対パスと .. は拒否され、この階層はオフのままになります。
admin-settings-key-privacy-image-disk-mode = 画像ディスクのサイズ調整方法
admin-settings-help-privacy-image-disk-mode = 自動では、このキャッシュが使える領域（空き領域とすでに保持している分の合計）の 10 分の 1 を、下限と上限の範囲内で、かつその領域の半分を超えないように使います。手動では下の最大サイズをそのまま使います。デフォルトは自動です。
admin-settings-key-privacy-image-disk-max-mi-b = 画像ディスクのサイズ（手動）
admin-settings-help-privacy-image-disk-max-mi-b = 手動モードでのディスク階層のサイズ（MiB）です。どちらのモードでも、表示には実際に有効なサイズが示されます。範囲は 1~1048576、デフォルトは 512 です。
admin-settings-key-privacy-image-disk-floor-mi-b = 自動ディスクサイズの下限
admin-settings-help-privacy-image-disk-floor-mi-b = 自動モードでのディスク階層の最小サイズ（MiB）で、空き領域が不明なときにも使われます。範囲は 0~1048576、デフォルトは 64 です。
admin-settings-key-privacy-image-disk-ceiling-mi-b = 自動ディスクサイズの上限
admin-settings-help-privacy-image-disk-ceiling-mi-b = 自動モードでのディスク階層の最大サイズ（MiB）です。範囲は 0~1048576、デフォルトは 4096 です。

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-privacy-image-disk-entries = ディスクエントリの上限
admin-settings-help-privacy-image-disk-entries = ディスクに保持する画像の最大数です。各エントリはインデックスに一定量の RAM も消費し、それは RAM 階層に計上されます。範囲は 16~1048576、デフォルトは 16384 です。
admin-settings-key-privacy-image-disk-entry-bytes = ディスク画像の最大サイズ
admin-settings-help-privacy-image-disk-entry-bytes = ディスクに書き込む、再エンコード後の画像 1 枚あたりの最大サイズ（バイト）です。RAM のエントリ上限を超えていてもこの値以内の画像は、直接ディスクに書き込まれます。範囲は 65536~67108864、デフォルトは 8388608 です。
admin-settings-key-privacy-image-disk-min-hits = ディスク書き込みに必要なヒット数
admin-settings-help-privacy-image-disk-min-hits = RAM のエントリが追い出されるときに、破棄ではなくディスクに書き込まれるために必要なヒット数です。0 にすると、スコアが降格閾値を超えた追い出し対象をすべて書き込みます。範囲は 0~100、デフォルトは 1 です。
admin-settings-key-privacy-image-disk-write-queue-mi-b = ディスク書き込みキュー
admin-settings-help-privacy-image-disk-write-queue-mi-b = バックグラウンドのディスク書き込み処理を待機できる画像データ量（MiB）です。RAM 階層に計上され、これを超えると追い出された画像はキューに入れずに破棄されます。範囲は 1~256、デフォルトは 8 です。
admin-settings-key-privacy-image-ttl-step-secs = ヒットごとの保持期間の加算
admin-settings-help-privacy-image-ttl-step-secs = ヒット時に画像の残り保持期間へ加算する秒数です。減衰したヒット数の対数に 1 を足した値を掛けるため、よく使われる画像ほど長く残ります。0 にすると、すべての画像が基本の保持期間のままになります。範囲は 0~86400、デフォルトは 600 です。
admin-settings-key-privacy-image-ttl-max-secs = 延長後の最長保持期間
admin-settings-help-privacy-image-ttl-max-secs = ヒットによって積み上げられる残り保持期間の最大値（秒）です。範囲は 60~2592000、デフォルトは 86400 です。
admin-settings-key-privacy-image-max-age-secs = 画像の最大保持期間
admin-settings-help-privacy-image-max-age-secs = ヒット数にかかわらず、この秒数を過ぎた画像は再取得されます。取得経路は上流の ETag や Last-Modified を保持しないため、条件付きの再検証は行わず、画像を単に取得し直します。範囲は 3600~7776000、デフォルトは 604800 です。
admin-settings-key-privacy-image-score-half-life-secs = スコアの半減期
admin-settings-help-privacy-image-score-half-life-secs = RAM、ディスク、追い出しを決める優先度スコアで、画像のヒット数と新しさが半分になるまでの秒数です。範囲は 60~604800、デフォルトは 3600 です。
admin-settings-key-privacy-image-score-demote-milli = 降格スコア
admin-settings-help-privacy-image-score-demote-milli = これを下回ると RAM の画像がディスクへ降格または破棄される、1000 分率のスコアです。昇格の閾値は、これにヒステリシスを加えた値になります。範囲は 0~10000、デフォルトは 300 です。
admin-settings-key-privacy-image-score-hysteresis-percent = 昇格のヒステリシス
admin-settings-help-privacy-image-score-hysteresis-percent = 昇格の閾値を降格の閾値よりどれだけ高くするかを示すパーセントです。両者の間のスコアでは、画像は現在の場所にとどまります。範囲は 0~500、デフォルトは 50 です。
admin-settings-key-privacy-image-context-open-percent = 開いているメッセージの重み
admin-settings-help-privacy-image-context-open-percent = メッセージ用に要求された画像のスコアの重み（パーセント）です。メッセージの画像は、10 分間操作がないと「最近」として扱われます。範囲は 100~1000、デフォルトは 200 です。
admin-settings-key-privacy-image-context-recent-percent = 最近のメッセージの重み
admin-settings-help-privacy-image-context-recent-percent = 最近使われた画像のスコアの重み（パーセント）です。メッセージを指定しないリクエストの重みは 100 です。範囲は 100~1000、デフォルトは 150 です。
admin-settings-key-privacy-image-account-share-percent = アカウントの RAM 占有率
admin-settings-help-privacy-image-account-share-percent = 他のアカウントがキャッシュを使っている間、1 つのアカウントの追加が RAM 階層に占められる最大の割合です。これを超えると、そのアカウントは自分のスコアが最も低い画像から先に追い出します。アカウントが 1 つだけの場合は、すべて使えます。範囲は 1~100、デフォルトは 50 です。
admin-settings-key-privacy-image-sweep-secs = キャッシュのスイープ間隔
admin-settings-help-privacy-image-sweep-secs = バックグラウンドのスイープの間隔（秒）です。スイープでは、画像の期限切れ処理、一定数のエントリの再スコア、負荷時の冷たい RAM 画像の降格、ディスクの空き容量の再確認を行います。範囲は 5~3600、デフォルトは 30 です。
admin-settings-key-privacy-image-snapshot-secs = インデックスのスナップショット間隔
admin-settings-help-privacy-image-snapshot-secs = ディスクインデックスをアトミックにスナップショットする間隔（秒）です。ヒットごとに書き込まなくても、ヒット数と延長された保持期間が再起動後も保たれます。0 にするとスナップショットがオフになり、再起動時に各ファイルのヘッダーを読み込みます。範囲は 0~86400、デフォルトは 300 です。
admin-settings-key-privacy-image-sweep-batch = スイープのバッチサイズ
admin-settings-help-privacy-image-sweep-batch = 1 回のメンテナンススイープで期限切れ処理、再スコア、追い出しができるエントリ数です。キャッシュロック中に行う作業量の上限になります。範囲は 16~8192、デフォルトは 256 です。
admin-image-proxy-title = 画像キャッシュの状況
admin-image-proxy-unavailable = 画像キャッシュの状況を取得できません。
admin-image-proxy-unknown = 不明
admin-image-proxy-ram = RAM 階層： { $budget } 中 { $used } 使用（設定値 { $configured }）、画像 { $entries } 枚、ホット { $hot }、コールド { $cold }、ヒット率 { $hit }。
admin-image-proxy-disk-off = ディスク階層： オフ。
admin-image-proxy-disk = ディスク階層： 実効 { $budget } 中 { $bytes } 使用（{ $mode }、下限 { $floor }、上限 { $ceiling }、手動 { $manual }）、画像 { $entries } 枚、ホット { $hot }、コールド { $cold }、ヒット率 { $hit }。
admin-image-proxy-disk-inactive = ディスク階層： オンですが有効ではありません。{ $reason }
admin-image-proxy-moves = 1 分あたりの昇格数： { $perMin }。昇格の合計 { $promotions }、降格 { $demotions }、破棄 { $drops }。インデックスのエントリ数： { $index }。
admin-image-proxy-refresh = 状況を更新
admin-settings-key-security-trust-enabled = 信頼ティア
admin-settings-help-security-trust-enabled = 各リクエストにティアを割り当てます。サインイン済み、既知の接続元（そのアドレスからの最近のサインインが成功している）、不明のいずれかです。変更されるのは匿名リクエストの許容量のみで、認証情報、第 2 要素、ロックアウトが緩和されることはありません。
admin-settings-key-security-trust-known-ttl-days = 既知の接続元の期間（日）
admin-settings-help-security-trust-known-ttl-days = サインインが成功してから、そのアドレスが既知として扱われる期間です。失敗した試行はカウントされません。
admin-settings-key-security-trust-known-capacity = 保持する既知の接続元の数
admin-settings-help-security-trust-known-capacity = メモリに保持する既知の接続元の最大数です（それぞれ、ソルト付きハッシュ、国、アカウント数を含みます）。最も古いものから破棄されます。
admin-settings-key-security-trust-persist = 再起動後も既知の接続元を保存
admin-settings-help-security-trust-persist = 既知の接続元の表（キー付きハッシュ、国、最終確認日時、アカウント数）を設定ストアに保存し、起動時に読み込みます。期間を過ぎたエントリは破棄されます。
admin-settings-key-security-trust-persist-interval-secs = 既知の接続元の保存間隔（秒）
admin-settings-help-security-trust-persist-interval-secs = 保存の間隔（秒）で、30 から 3600 までです。表が変更されたときのみ保存され、正常なシャットダウン時にもう一度保存されます。
admin-settings-key-security-trust-unknown-percent = 不明な接続元の許容量（%）
admin-settings-help-security-trust-unknown-percent = 設定済みの制限に対する、不明な接続元の匿名リクエスト許容量の割合です。100 で基準のまま、低いほど厳しくなります。
admin-settings-key-security-trust-known-percent = 既知の接続元の許容量（%）
admin-settings-help-security-trust-known-percent = 既知の接続元の匿名リクエスト許容量です。200 にするとリクエストの予算が 2 倍になり、サインインのクールダウンとチャレンジの難易度が半分になります。認証情報、第 2 要素、ロックアウトを回避することはありません。
admin-settings-key-security-trust-ua-missing = ボット： 空の User-Agent
admin-settings-help-security-trust-ua-missing = User-Agent が空の匿名リクエストに対する動作です。許可、制限、遮断から選びます。サインイン済みのセッションには影響しません。
admin-settings-key-security-trust-ua-uncommon = ボット： 未認識の User-Agent
admin-settings-help-security-trust-ua-uncommon = 認識できない User-Agent の匿名リクエストに対する動作です。許可、制限、遮断から選びます。
admin-settings-key-security-trust-ua-automation = ボット： スクリプトと自動化
admin-settings-help-security-trust-ua-automation = スクリプト、HTTP ライブラリ、ヘッドレスブラウザー、汎用ボットを名乗る匿名リクエストに対する動作です。許可、制限、遮断から選びます。
admin-settings-key-security-trust-ua-search = ボット： 検索クローラー
admin-settings-help-security-trust-ua-search = 検索エンジンのクローラーを名乗る匿名リクエストに対する動作です。許可、制限、遮断から選びます。
admin-settings-key-security-trust-ua-ai-training = ボット： AI 学習用クローラー
admin-settings-help-security-trust-ua-ai-training = GPTBot、ClaudeBot、CCBot、Amazonbot、Bytespider、Google-Extended、Applebot-Extended を名乗る匿名リクエストに対する動作です。許可、制限、遮断から選びます。名乗りはヘッダーにすぎず、検証済みの ID ではありません。
admin-settings-key-security-trust-ua-ai-search = ボット： AI 検索クローラー
admin-settings-help-security-trust-ua-ai-search = OAI-SearchBot、PerplexityBot、Claude-SearchBot を名乗る匿名リクエストに対する動作です。許可、制限、遮断から選びます。
admin-settings-key-security-trust-ua-ai-user = ボット： AI ユーザー取得
admin-settings-help-security-trust-ua-ai-user = ChatGPT-User または Claude-User を名乗る匿名リクエストに対する動作です。許可、制限、遮断から選びます。
admin-settings-key-security-trust-ua-custom = ボット： カスタムパターン
admin-settings-help-security-trust-ua-custom = カスタムの User-Agent パターンに一致する匿名リクエストに対する動作です。許可、制限、遮断から選びます。
admin-settings-key-security-trust-admin-geo-enabled = 不明な国からの管理画面探索を遮断
admin-settings-help-security-trust-admin-geo-enabled = 最近のサインインが成功していない国から管理コンソールを探索する接続元を遮断します。読み取り可能な国データベースと、管理画面探索の保護が必要です。復旧のために MW_ADMIN_GEO_ENABLED=false でオフにできます。
admin-settings-key-security-trust-admin-geo-countries = 常に既知とする国（管理画面の遮断）
admin-settings-help-security-trust-admin-geo-countries = 最近のサインインが成功した国に加えて、常に既知として扱う大文字の ISO 国コードです。
admin-settings-key-security-trust-admin-geo-lookup-failure = 国が不明な接続元
admin-settings-help-security-trust-admin-geo-lookup-failure = allow は遮断せずにその旨を記録します。unknown は国が検出されない接続元を不明な国として扱います。
admin-settings-key-security-trust-admin-geo-threshold = 遮断までの探索回数
admin-settings-help-security-trust-admin-geo-threshold = ウィンドウ内に不明な国から行われた管理画面探索が、この回数に達すると遮断を開始します。
admin-settings-key-security-trust-admin-geo-window-secs = 探索ウィンドウ（秒）
admin-settings-help-security-trust-admin-geo-window-secs = 探索の閾値に使うウィンドウです。
admin-settings-key-security-trust-admin-geo-block-on-first = 最初の探索で遮断
admin-settings-help-security-trust-admin-geo-block-on-first = 不明な国からの最初の探索で遮断します。出張中の管理者を締め出す可能性が高まります。
admin-settings-key-security-trust-admin-geo-block-secs = 1 回目の遮断（秒）
admin-settings-help-security-trust-admin-geo-block-secs = 1 回目の遮断の長さです。
admin-settings-key-security-trust-admin-geo-second-block-secs = 2 回目の遮断（秒）
admin-settings-help-security-trust-admin-geo-second-block-secs = 繰り返し期間内に同じ接続元が 2 回目に遮断される長さです。
admin-settings-key-security-trust-admin-geo-max-block-secs = 繰り返しの遮断（秒）
admin-settings-help-security-trust-admin-geo-max-block-secs = 繰り返し期間内の 3 回目以降のすべての遮断の長さです。
admin-settings-key-security-trust-admin-geo-repeat-days = 繰り返し期間（日）
admin-settings-help-security-trust-admin-geo-repeat-days = 前回の遮断からこの日数以内の遮断は繰り返しとみなされ、次の長い時間が適用されます。
admin-settings-key-security-signin-account-geo-mode = アカウント別のサインイン国ポリシー
admin-settings-help-security-signin-account-geo-mode = デフォルトではオフです。アカウントがこれまでに使っていない国、または許可リスト外の国からのサインインを遮断します。許可リストにある IP と国は常に通過します。適用対象は新しいウェブメールのサインインのみで、既存のセッションやプロトコルクライアントには適用されません。国の判定には IP 位置情報データベースを使うため、誤りや古い情報が含まれる場合があります。VPN、モバイル通信事業者、プロキシ、旅行などにより、正当なサインインが遮断されたり、海外からのサインインが通過したりすることがあります。信頼できる IP や国は許可リストに追加してください。
admin-settings-key-security-signin-account-geo-allow-countries = アカウントで許可する国
admin-settings-help-security-signin-account-geo-allow-countries = アカウントで常に許可する大文字の ISO 国コードです。国の判定には IP 位置情報データベースを使うため、誤りや古い情報が含まれる場合があります。VPN、モバイル通信事業者、プロキシ、旅行などにより、正当なサインインが遮断されたり、海外からのサインインが通過したりすることがあります。信頼できる IP や国は許可リストに追加してください。
admin-settings-key-security-signin-account-geo-allow-ips = アカウントで許可する IP
admin-settings-help-security-signin-account-geo-allow-ips = アカウントで常に許可する IP アドレスまたは CIDR ネットワークです。単一の IPv6 アドレスは、その /64 を意味します。
admin-settings-key-security-signin-account-geo-global-allow-countries = すべてのアカウントで許可する国
admin-settings-help-security-signin-account-geo-global-allow-countries = すべてのアカウントで許可する大文字の ISO 国コードで、各アカウント自身のリストに追加されます。
admin-settings-key-security-signin-account-geo-global-allow-ips = すべてのアカウントで許可する IP
admin-settings-help-security-signin-account-geo-global-allow-ips = すべてのアカウントで許可する IP アドレスまたは CIDR ネットワーク（オフィスや VPN など）で、各アカウント自身のリストに追加されます。
admin-settings-key-security-signin-account-geo-lookup-failure = 国が不明な接続元（アカウントポリシー）
admin-settings-help-security-signin-account-geo-lookup-failure = 接続元の国が検出されないとき、サインインを許可して記録するか、拒否するかを選びます。
admin-settings-key-security-signin-account-geo-history-days = アカウントの国の履歴（日）
admin-settings-help-security-signin-account-geo-history-days = 通常と異なる国のポリシーのために、アカウント自身のサインイン履歴に国を残す日数です。認証ログとは別に保持されるため、ログを短くしてもこの期間は短くなりません。
admin-settings-key-security-geo-no-database = 国データベースなし： デフォルトの動作
admin-settings-help-security-geo-no-database = データベースが読み込まれていないときの国関連機能の動作です。許可、接続元を不明な国として扱う、遮断（遮断はサインインにのみ適用）から選びます。各機能自身の設定も適用され、より厳しい方が優先されます。
admin-settings-key-security-geo-not-found = データベースにないアドレス： デフォルトの動作
admin-settings-help-security-geo-not-found = データベースは読み込まれているが、そのアドレスのエントリがないときの国関連機能の動作です。プライベート、ループバック、リンクローカルのアドレスはローカルネットワークであり、影響を受けません。
admin-settings-key-security-signin-geo-no-database = データベースがない場合のインスタンスのサインインポリシー
admin-settings-help-security-signin-geo-no-database = インスタンスのサインイン国ポリシーがオンでデータベースが読み込まれていないとき、サインインを遮断する（フェイルクローズ）か、チェックせずに許可するかを選びます。ローカルネットワークは常に許可されます。
admin-settings-key-security-geo-session-lookup-failure = 国が不明なセッションのアラート
admin-settings-help-security-geo-session-lookup-failure = サインイン済みリクエストの国を判定できないとき、無視するか、セッションごとに 1 日 1 回セッションアラートを出すかを選びます。セッションを遮断したり失効させたりすることはありません。
admin-settings-key-security-protocol-framing-mode = プロトコルガード： リクエストのフレーミング
admin-settings-help-security-protocol-framing-mode = リクエストスマグリングに分類される問題です。Content-Length と Transfer-Encoding の併用、chunked 以外（または HTTP/1.0 や HTTP/2 上）の Transfer-Encoding、数値でない、または矛盾する Content-Length が対象です。強制では 400 を返して接続を閉じ、検出のみでは数えるだけです。ルートプレフィックスによる除外は、この分類には適用されません。
admin-settings-key-security-protocol-host-mode = プロトコルガード： Host の構文
admin-settings-help-security-protocol-host-mode = Host ヘッダーの構文です。HTTP/1.1 での欠落、空、重複、またはユーザー情報、空白、パス文字を含むものが対象です。IPv6 リテラル、ポート、末尾のドットは受け入れられます。強制では 400 を返して接続を閉じます。
admin-settings-key-security-protocol-host-pinning-mode = プロトコルガード： Host は公開名と一致する必要がある
admin-settings-help-security-protocol-host-pinning-mode = Host が MW_PUBLIC_URL のホストまたは allowedHosts のいずれとも一致しない場合です。どちらもホストを指定していない間は何もしません。プロキシやヘルスチェックが内部名を送る場合があるため、デフォルトは検出のみです。
admin-settings-key-security-protocol-path-control-mode = プロトコルガード： パス内の制御文字
admin-settings-help-security-protocol-path-control-mode = リクエストパス内の NUL、制御文字、エンコードされた CR/LF（%00、%0d、%0a）です。強制では 400 を返します。ルートプレフィックスによる除外の対象にはなりません。
admin-settings-key-security-protocol-target-mode = プロトコルガード： リクエストターゲットの細工
admin-settings-help-security-protocol-target-mode = ドットとスラッシュの二重または過長なパーセントエンコード、エンコードされたドット、ドットセグメント、空のパスセグメント（//）、バックスラッシュ、生の非 ASCII バイト、Host とは別のオーソリティを指す絶対形式のターゲットが対象です。
admin-settings-key-security-protocol-path-length-mode = プロトコルガード： パスとクエリの長さ
admin-settings-help-security-protocol-path-length-mode = パスまたはクエリが maxPathBytes または maxQueryBytes より長い場合です。強制では 414 を返します。
admin-settings-key-security-protocol-method-mode = プロトコルガード： HTTP メソッド
admin-settings-help-security-protocol-method-mode = allowedMethods にないメソッドです（デフォルトでは TRACE、TRACK、CONNECT、および未知または小文字のメソッド）。強制では 405 を返します。
admin-settings-key-security-protocol-duplicate-headers-mode = プロトコルガード： 重複した単一値ヘッダー
admin-settings-help-security-protocol-duplicate-headers-mode = 1 回だけ現れるべきヘッダーが複数回現れる場合（Content-Type、Authorization、Origin、Referer、User-Agent、Sec-Fetch-*、条件付きヘッダーと範囲ヘッダー）、同じ値の Content-Length が重複している場合、または HTTP/1.x で Cookie ヘッダーが重複している場合です。
admin-settings-key-security-protocol-forwarded-mode = プロトコルガード： 信頼できないピアからの転送ヘッダー
admin-settings-help-security-protocol-forwarded-mode = MW_TRUSTED_PROXIES 以外のピアが送信した Forwarded、X-Forwarded-*、X-Real-IP です。これらのヘッダーはどちらの場合も信頼されず、この設定は試行を記録します。適応型保護に計上されるのは、信頼できるプロキシが設定されている場合のみです。
admin-settings-key-security-protocol-header-hygiene-mode = プロトコルガード： ヘッダーの健全性
admin-settings-help-security-protocol-header-hygiene-mode = ヘッダー数、単一の値、または Cookie ヘッダーが設定された上限を超えている場合、非 ASCII のヘッダーバイト、WebSocket ルート以外での Upgrade または Connection: upgrade、Content-Length や Host などのヘッダーを指す Connection が対象です。強制では、サイズと数に関する検出には 431、それ以外には 400 を返します。
admin-settings-key-security-protocol-body-mode = プロトコルガード： リクエストボディのフレーミング
admin-settings-help-security-protocol-body-mode = ボディを持つ GET または HEAD、maxBodyBytes を超える Content-Length（413）、100-continue 以外の Expect（417）、またはボディのない Expect が対象です。
admin-settings-key-security-protocol-max-path-bytes = プロトコルガード： パスの最大長（バイト）
admin-settings-help-security-protocol-max-path-bytes = パス長の分類が報告するまでに許容するリクエストパスの最大長です。
admin-settings-key-security-protocol-max-query-bytes = プロトコルガード： クエリの最大長（バイト）
admin-settings-help-security-protocol-max-query-bytes = パス長の分類が報告するまでに許容するクエリ文字列の最大長です。
admin-settings-key-security-protocol-max-header-count = プロトコルガード： ヘッダーの最大数
admin-settings-help-security-protocol-max-header-count = ヘッダーの健全性の分類が報告するまでに許容する、1 リクエスト内のヘッダーフィールドの最大数です。リスナー自身のハードリミット（MW_HTTP_MAX_HEADERS）も引き続き適用されます。
admin-settings-key-security-protocol-max-header-value-bytes = プロトコルガード： ヘッダー値の最大長（バイト）
admin-settings-help-security-protocol-max-header-value-bytes = ヘッダーの健全性の分類が報告するまでに許容する、単一のヘッダー値の最大長です。
admin-settings-key-security-protocol-max-cookie-bytes = プロトコルガード： Cookie ヘッダーの最大サイズ（バイト）
admin-settings-help-security-protocol-max-cookie-bytes = ヘッダーの健全性の分類が報告するまでに許容する、Cookie ヘッダーの合計サイズです。
admin-settings-key-security-protocol-max-body-bytes = プロトコルガード： 宣言されたボディの最大サイズ（バイト）
admin-settings-help-security-protocol-max-body-bytes = ボディの分類が報告せずに受け入れる Content-Length の最大値です。ルートが受け付ける最大のアップロードやインポートより大きくしてください。ルートプレフィックスの除外で、特定のルートだけ引き上げることもできます。
admin-settings-key-security-protocol-allowed-methods = プロトコルガード： 許可するメソッド
admin-settings-help-security-protocol-allowed-methods = メソッドの分類が受け入れるメソッドです。デフォルトでは通常の HTTP メソッドと、WebDAV、CalDAV、CardDAV のメソッドが並んでいます。現在、サーバーは受信用の DAV ルートをマウントしていないため、これらはルーターで 405 になります。
admin-settings-key-security-protocol-allowed-hosts = プロトコルガード： 追加で受け入れるホスト
admin-settings-help-security-protocol-allowed-hosts = MW_PUBLIC_URL のホストに加えて、Host 固定の分類が受け入れるホスト名です。たとえば、プロキシやヘルスチェックが使う内部名などです。ポートと末尾のドットは無視されます。
admin-settings-key-security-protocol-exempt-prefixes = プロトコルガード： 除外するルートプレフィックス
admin-settings-help-security-protocol-exempt-prefixes = ヒューリスティックな分類を実行しないルートプレフィックスです（それぞれ / で始まり、2 文字以上）。大きなアップロード用ルートなど、必要とする正当なクライアント向けです。パスセグメント全体で一致します。フレーミングとパス制御の検出は除外されることがなく、/ は無視されます。
admin-settings-key-security-health-mode = ヘルスエンドポイントへのアクセス
admin-settings-help-security-health-mode = /healthz と /readyz にアクセスできる対象です。internal-only: ループバック、プライベート、リンクローカル、ユニークローカルのクライアントアドレス。allowlist: それらに加えて下記のネットワーク。open: 誰でも。それ以外は、未知のパスと同じ 404 になります。MW_HEALTH_TOKEN のトークンを付けたリクエストは常に許可されます。クライアントアドレスは、信頼できるプロキシ経由で解決されたものです。MW_FORWARDED_MODE と MW_TRUSTED_PROXIES のないリバースプロキシの背後では、すべてのリクエストがプロキシのプライベートアドレスに見えるため、インターネット全体が許可されます。
admin-settings-key-security-health-networks = ヘルスエンドポイントのネットワーク
admin-settings-help-security-health-networks = allowlist モードで、内部範囲に加えて /healthz と /readyz にアクセスできる IP アドレスまたは CIDR ネットワークです。たとえば、外部の稼働監視などです。
admin-settings-key-security-spray-enabled = パスワードスプレー検出
admin-settings-help-security-spray-enabled = 1 つの接続元がサインインに失敗したときの、異なるユーザー名の数をカウントします。各ユーザー名のキー付きハッシュのみがメモリ内に保持されます。
admin-settings-key-security-spray-window-secs = スプレーウィンドウ（秒）

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-security-spray-window-secs = 1 つの接続元のサインイン失敗を、異なるユーザー名の数でカウントするウィンドウです。デフォルトは 900 です。
admin-settings-key-security-spray-challenge-at = スプレー： チャレンジ閾値
admin-settings-help-security-spray-challenge-at = ウィンドウ内で 1 つの接続元から失敗した異なるユーザー名の数がこの値に達すると、その接続元にプルーフオブワークのチャレンジを強制します。成功したサインインはカウントされないため、共有のオフィスやキャンパスのアドレスは影響を受けません。デフォルトは 10 です。
admin-settings-key-security-spray-pace-at = スプレー： ペース制御の閾値
admin-settings-help-security-spray-pace-at = 失敗した異なるユーザー名の数がこの値に達すると、その接続元からの試行の間隔が長くなります。デフォルトは 15 です。
admin-settings-key-security-spray-lock-at = スプレー： ロックの閾値
admin-settings-help-security-spray-lock-at = 失敗した異なるユーザー名の数がこの値に達すると、既存のサインインロックを通じて、その接続元をロック時間のあいだロックします。デフォルトは 25 です。
admin-settings-key-security-spray-pace-secs = スプレー： ペース制御の待ち時間（秒）
admin-settings-help-security-spray-pace-secs = ペース制御の閾値以上になった接続元からの試行の間隔（秒）です。デフォルトは 10 です。
admin-settings-key-security-spray-lock-secs = スプレー： ロック時間（秒）
admin-settings-help-security-spray-lock-secs = 一時ロックの長さです。管理者が接続元のロックを解除すると、そのカウントもリセットされます。デフォルトは 900 です。
admin-settings-key-security-spray-known-multiplier = スプレー： 既知の接続元の倍率
admin-settings-help-security-spray-known-multiplier = 既知の接続元（最近のサインインが成功している接続元）に対する、3 つの閾値の倍率です。デフォルトは 3 です。
admin-settings-key-security-spray-capacity = スプレー： 追跡する接続元の数
admin-settings-help-security-spray-capacity = メモリ内で追跡する接続元の最大数です。最後のアクティビティが最も古いものから破棄されます。デフォルトは 20000 です。
admin-settings-key-security-spike-enabled = インスタンス全体の急増検出
admin-settings-help-security-spike-enabled = サインイン失敗の基準値を学習し、インスタンス全体が基準値を大きく上回ると警戒モードに入ります。インスタンスをロックすることはありません。
admin-settings-key-security-spike-multiplier-percent = 急増： 倍率(%)
admin-settings-help-security-spike-multiplier-percent = シグナルが、学習した基準値のこのパーセント以上（400 は 4 倍）で、かつ絶対的な下限を超えたときにカウントされます。
admin-settings-key-security-spike-min-failures-per-min = 急増： 1 分あたりの最小失敗数
admin-settings-help-security-spike-min-failures-per-min = 下限です。1 分あたりのサインイン失敗数がこれを下回ると、失敗率のシグナルはカウントされません。デフォルトは 20 です。
admin-settings-key-security-spike-min-sources = 急増： 失敗した接続元の最小数
admin-settings-help-security-spike-min-sources = 下限です。5 分間のバケット内で失敗した異なる接続元の数がこれを下回ると、そのシグナルはカウントされません。デフォルトは 10 です。
admin-settings-key-security-spike-min-usernames = 急増： 失敗したユーザー名の最小数
admin-settings-help-security-spike-min-usernames = 下限です。5 分間のバケット内で失敗した異なるユーザー名の数がこれを下回ると、そのシグナルはカウントされません。デフォルトは 30 です。
admin-settings-key-security-spike-signals-required = 急増： 必要なシグナル数
admin-settings-help-security-spike-signals-required = 基準値と下限を同時に超える必要があるシグナルの数です。分散型スプレーの特徴は 2 つとして数えられます。デフォルトは 2 です。
admin-settings-key-security-spike-warmup-hours = 急増： ウォームアップ（時間）
admin-settings-help-security-spike-warmup-hours = 警戒モードが開始できるようになるまでに観測するトラフィックの時間数です。それまでは基準値の学習のみを行います。デフォルトは 48 です。
admin-settings-key-security-spike-adapt-hours = 急増： 基準値の適応（時間）
admin-settings-help-security-spike-adapt-hours = 基準値の時定数です。長いほど増加をゆっくり取り込みます。急増によって基準値が更新されることはありません。デフォルトは 168 です。
admin-settings-key-security-spike-duration-secs = 急増： 警戒モードの継続時間（秒）
admin-settings-help-security-spike-duration-secs = 急増の最後の証拠から警戒モードが続く時間です。シグナルがトリガー水準の半分を上回っている間は延長されます。デフォルトは 1800 です。
admin-settings-key-security-spike-force-challenge = 急増： チャレンジを強制
admin-settings-help-security-spike-force-challenge = 警戒モードでは、不明な接続元からの匿名サインインにプルーフオブワークのチャレンジを求めます。認証済みユーザーと既知の接続元には影響しません。
admin-settings-key-security-spike-pace-factor-percent = 急増： サインインの待ち時間(%)
admin-settings-help-security-spike-pace-factor-percent = 警戒モードでの、不明な接続元に対するサインインの待ち時間を、通常の待ち時間に対するパーセントで示します。デフォルトは 300 です。
admin-settings-key-security-session-binding-enabled = セッションバインド
admin-settings-help-security-session-binding-enabled = 各ブラウザーセッションを、クライアント属性のキー付きハッシュに紐づけます。ベアラートークンを使うネイティブクライアント、API キー、プロトコルクライアントは対象外です。
admin-settings-key-security-session-binding-attributes = セッションバインド： 属性
admin-settings-help-security-session-binding-attributes = 紐づける属性は browser、os、hints、language、country、network（IPv4 は /16、IPv6 は /48）です。モバイルネットワークは変動するため、network はデフォルトでオフです。不明な名前は無視されます。
admin-settings-key-security-session-binding-strictness = セッションバインド： 厳格さ
admin-settings-help-security-session-binding-strictness = 寛容ではブラウザーの更新と、ソフトな属性 1 つの変化を許容し、厳格ではソフトな属性の違いをすべて検出します。
admin-settings-key-security-session-binding-webmail-action = セッションバインド： ウェブメールの動作
admin-settings-help-security-session-binding-webmail-action = ウェブメールのセッションが一致しないときの動作です。off、detect（記録のみ）、step-up（パスワードと第 2 要素を再度要求）、またはセッションの失効から選びます。デフォルトは step-up です。
admin-settings-key-security-session-binding-admin-action = セッションバインド： 管理コンソールの動作
admin-settings-help-security-session-binding-admin-action = 管理コンソールのセッションが一致しないときの動作です。off、detect、step-up、または失効から選びます。デフォルトは失効です。
admin-settings-key-security-theft-enabled = セッション窃取の検出
admin-settings-help-security-theft-enabled = ローテーション済みセッション ID の再利用、あり得ない移動、異なる国からの同時使用を検出します。
admin-settings-key-security-theft-rotation-grace-secs = 窃取： ローテーションの猶予（秒）
admin-settings-help-security-theft-rotation-grace-secs = セッションがローテーションされた後、古い ID がこの時間のあいだ再び提示されても（処理中のリクエスト）、窃取とはみなされません。デフォルトは 30 です。
admin-settings-key-security-theft-min-travel-secs = 窃取： 最小移動時間（秒）
admin-settings-help-security-theft-min-travel-secs = 前回のリクエストからこれより短い時間で国が変わると、あり得ない移動として扱われます。国の判定は不正確な場合があり、VPN やモバイルネットワークにより誤った変化が生じることがあります。デフォルトは 3600 です。
admin-settings-key-security-theft-concurrent-use = 窃取： 同時使用
admin-settings-help-security-theft-concurrent-use = 5 分以内に 1 つのセッションが異なる国（またはネットワーク）から交互に使われたときに検出します。off、country、network から選びます。国の判定は不正確な場合があり、VPN やモバイルネットワークにより誤った変化が生じることがあります。
admin-settings-key-security-theft-webmail-action = 窃取： ウェブメールでの対応
admin-settings-help-security-theft-webmail-action = ウェブメールのセッションで、あり得ない移動や同時使用があったときの対応です。alert、step-up、失効から選びます。ローテーション済みセッションの再利用は、常に失効されます。デフォルトは step-up です。
admin-settings-key-security-theft-admin-action = 窃取： 管理コンソールでの対応
admin-settings-help-security-theft-admin-action = 管理コンソールのセッションで、あり得ない移動や同時使用があったときの対応です。alert、step-up、失効から選びます。デフォルトは失効です。
admin-settings-key-security-theft-retention-days = 窃取： 記録の保持期間（日）
admin-settings-help-security-theft-retention-days = ローテーション済みセッションのマーカーと、セッションごとのセキュリティ記録を保持する期間です。最長のセッション有効期間以上に設定してください。デフォルトは 15 です。
admin-settings-key-security-alerts-country-change = 国の変更アラート
admin-settings-help-security-alerts-country-change = サインイン済みのセッションが、開始された国とは別の国から使われたときに警告します。国の判定は不正確な場合があり、VPN やモバイルネットワークにより誤った変化が生じることがあります。ロックされていない限りユーザーが変更できます。ユーザーがオフにした設定は、ミュートが許可されている場合にのみ反映されます。
admin-settings-key-security-alerts-country-change-enabled = 国の変更アラート： 全体のスイッチ
admin-settings-help-security-alerts-country-change-enabled = オフにすると、全員に対して国の変更アラートが無効になります。国の判定は不正確な場合があり、VPN やモバイルネットワークにより誤った変化が生じることがあります。
admin-settings-key-security-alerts-country-change-cooldown-secs = 国の変更アラート： クールダウン（秒）
admin-settings-help-security-alerts-country-change-cooldown-secs = 1 つのセッションにおける、国の変更アラートの最小間隔です。新しい国ごとに、セッションあたり 1 回通知します。デフォルトは 300 です。
admin-settings-key-security-alerts-country-change-user-mute = 国の変更アラート： ユーザーによるミュートを許可
admin-settings-help-security-alerts-country-change-user-mute = ユーザーが自分の国の変更アラートをオフにできるかどうかです。オフの場合は、ユーザー自身の設定がオフでもサーバーは通知を続けます。
admin-settings-key-security-alerts-channels = セキュリティアラートのチャネル
admin-settings-help-security-alerts-channels = セッションのセキュリティ警告をユーザーに届ける方法です。アプリ内、ユーザー自身のメールボックスへのメッセージ、またはその両方から選びます。ロックされていない限りユーザーが変更できます。
admin-settings-key-security-alerts-email-enabled = メールによるセキュリティアラート
admin-settings-help-security-alerts-email-enabled = セッションのセキュリティ警告を、ユーザー自身のメールボックスに配信できるようにします。SMTP でメッセージが送信されることはなく、配信セッションがない場合、警告はアプリ内と監査ログに残ります。
admin-settings-key-security-alerts-sensitivity = セキュリティアラートの感度
admin-settings-help-security-alerts-sensitivity = 「低」は位置情報に関する検出のみ警告し、追加認証や失効は行いません。「標準」は管理者の動作設定に従います。「高」は移動の間隔を 2 倍にし、同時使用のネットワークも確認します。ロックされていない限りユーザーが変更できます。
admin-settings-key-security-alerts-notice-retention-days = セキュリティ通知： 保持期間（日）
admin-settings-help-security-alerts-notice-retention-days = アプリ内のセキュリティ通知を、アカウントごとに保持する期間です（アカウントあたり最大 20 件）。デフォルトは 30 です。
admin-settings-inline-effective = 現在の設定：
admin-settings-inline-reset = 変更を破棄
admin-settings-inline-domains = ドメインごとの値
admin-attachments-tab-lists = 許可リストとブロックリスト
admin-attachments-tab-limits = 制限
admin-attachments-tab-delivery = ダウンロードとプレビュー
admin-attachments-tab-warnings = 警告
admin-attachments-tab-test = ファイルをテスト
admin-attachments-tab-denied = このタブにアクセスする権限がありません。
admin-attachments-group-lists = 許可リストとブロックリスト
admin-attachments-group-limits = サイズ制限
admin-attachments-group-delivery = ダウンロードとプレビュー
admin-attachments-group-warnings = 警告
admin-attachments-add-extension = 拡張子（例： pdf または .tar.gz）
admin-attachments-add-mime = MIME タイプ（例： image/png または image/*）
admin-attachments-add = 追加
admin-attachments-remove = { $rule } を削除
admin-attachments-filter = リストを絞り込む
admin-attachments-list-label = このリストのルール
admin-attachments-count = { $max } 件中 { $count } 件のルール
admin-attachments-empty-list = このリストにルールはありません。
admin-attachments-none-match = 絞り込みに一致するルールはありません。
admin-attachments-bulk = まとめて貼り付け
admin-attachments-bulk-add = すべて追加
admin-attachments-import = テキストファイルをインポート
admin-attachments-export = テキストとしてエクスポート
admin-attachments-err-extension = 拡張子ではありません。英字、数字、ハイフン、アンダースコアを使い、32 文字までにしてください。tar.gz のように、ドットで区切ることもできます。
admin-attachments-err-mime = MIME タイプではありません。type/subtype の形式、またはすべてのサブタイプを表す type/* を使ってください。
admin-attachments-err-long = そのルールは長すぎます。
admin-attachments-invalid = 追加されませんでした。有効なルールではありません： { $entries }
admin-attachments-duplicates = すでにリストにあります： { $rules }
admin-attachments-overflow = 1 つのリストに入れられるルールは最大 { $max } 件です。残りは追加されませんでした。
admin-attachments-conflict-blockedOverAllowed = { $rule } は、もう一方のリストの { $other } と重複しています。ブロックのルールが優先されるため、ブロックされます。
admin-attachments-conflict-blockedOverWarn = { $rule } は、もう一方のリストの { $other } と重複しています。ブロックされたファイルは警告より前に拒否されるため、警告は表示されません。
admin-attachments-conflict-allowedAndWarn = { $rule } は、もう一方のリストの { $other } と重複しています。ファイルは許可され、警告が表示されます。
admin-attachments-bytes-unit = 単位
admin-attachments-unlimited = サイズ制限なし
admin-attachments-bytes-equals = { $size }（{ $bytes } バイト）
admin-attachments-list-allowedExtensions = 許可する拡張子
admin-attachments-list-allowedMimeTypes = 許可する MIME タイプ
admin-attachments-list-blockedExtensions = ブロックする拡張子
admin-attachments-list-blockedMimeTypes = ブロックする MIME タイプ
admin-attachments-list-warnExtensions = 警告する拡張子
admin-attachments-list-warnMimeTypes = 警告する MIME タイプ
admin-attachments-sim-title = ファイルをテスト
admin-attachments-sim-help = 保存済みのポリシーが、この名前、タイプ、サイズのファイルをどう扱うかをサーバーに問い合わせます。何も変更も保存もされません。メッセージや添付ファイルが読み取られることはありません。
admin-attachments-sim-filename = ファイル名
admin-attachments-sim-mime = MIME タイプ
admin-attachments-sim-size = サイズ
admin-attachments-sim-run = テスト
admin-attachments-sim-invalid = ファイル名と、0 以上のサイズを入力してください。
admin-attachments-sim-allowed = 許可
admin-attachments-sim-warn = 警告付きで許可
admin-attachments-sim-blocked = 配信されません
admin-attachments-sim-preview-yes = プレビューは許可されています。
admin-attachments-sim-preview-no = プレビューは許可されていません。
admin-attachments-sim-checked = 確認したサイズ： { $size }。
admin-attachments-sim-matched = 一致したルールのあるリスト： { $lists }。
admin-attachments-sim-matched-none = 一致する許可、ブロック、警告のルールはありません。
admin-attachments-reason-downloadsDisabled = ダウンロードはオフになっています。
admin-attachments-reason-attachmentTooLarge = ファイルがサイズ制限を超えています。
admin-attachments-reason-attachmentBlocked = ブロックのルールに一致しています。
admin-attachments-reason-attachmentNotAllowed = 許可リストがあり、その中に一致するものがありません。
admin-attachments-reason-attachmentContainerRestricted = 内容が一覧にないため、制限がオンの間は、添付されたメッセージと TNEF ファイルは拒否されます。
admin-settings-key-mail-reply-images = 返信と転送の画像
admin-settings-help-mail-reply-images = 返信や転送に元のメッセージの画像を含めるかどうかです。確認する（デフォルト）、常に許可、信頼できる送信者ドメインのみ許可して他は確認する、常に拒否から選びます。この設定をロックすると強制されます。画像は含める場合にのみ読み込まれます。
admin-settings-key-mail-reply-image-trusted-domains = 信頼できる画像の送信者ドメイン
admin-settings-help-mail-reply-image-trusted-domains = ポリシーが信頼できるドメインのとき、確認なしで画像を含める送信者ドメイン（完全一致）を最大 200 件まで登録できます。この設定がロックされていない限り、ユーザーは引用の確認から自分のリストに追加できます。
admin-settings-key-mail-send-recall-delay-seconds = 送信取り消しの猶予（デフォルト）
admin-settings-help-mail-send-recall-delay-seconds = 送信したメッセージが、配信されるまで送信トレイで待機し、送信者が取り消せる秒数です。0 の場合はすぐに送信します。デフォルトは 10 で、最大 1800（30 分）です。この設定をロックしない限り、ユーザーは下の上限までの値を自分で選べます。
admin-settings-key-mail-send-recall-delay-max-seconds = 送信取り消しの猶予（上限）
admin-settings-help-mail-send-recall-delay-max-seconds = ユーザーが選べる送信取り消しの猶予の最大値（秒）です。これより長い指定は、サーバーがこの値に短縮します。デフォルトと最大値は 1800 です。
admin-settings-key-mail-send-allow-recall = 送信済みメッセージの取り消しを許可

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-send-allow-recall = オフにすると、メッセージはすぐに送信され、送信トレイから取り消せなくなります。予約送信は影響を受けず、予定の時刻までは引き続きキャンセルできます。
admin-settings-key-mail-send-max-delayed-seconds = 予約送信の上限期間
admin-settings-help-mail-send-max-delayed-seconds = メッセージを何秒先まで予約できるかです。デフォルトは 7776000（90 日）、最大は 31536000（1 年）です。0 にすると予約送信がオフになります。
admin-settings-key-mail-send-max-queued = メールボックスごとのキュー内メッセージ数
admin-settings-help-mail-send-max-queued = 1 つのメールボックスで、送信トレイ（取り消し猶予中または予約済み）に同時に待機できるメッセージ数です。デフォルトは 200 です。これを超える送信は、明確なメッセージとともに拒否され、破棄されることはありません。
admin-attachments-sim-pick = ファイルから入力（任意）
admin-attachments-mime-toggle = よく使う MIME タイプを表示
admin-attachments-mime-hint = よく使うタイプを選ぶか、任意の type/subtype を入力してください。
admin-attachments-mime-invalid = text/plain のような type/subtype ではありません。入力したとおりに送信されます。
admin-attachments-mime-group-documents = ドキュメント
admin-attachments-mime-group-images = 画像
admin-attachments-mime-group-archives = アーカイブ
admin-attachments-mime-group-executables = 実行ファイルとスクリプト
admin-attachments-mime-group-media = 音声と動画
admin-attachments-mime-group-email = メールとカレンダー
admin-attachments-mime-group-other = その他
admin-settings-key-cookies-enabled = Cookie 同意コントロール
admin-settings-help-cookies-enabled = 訪問者に Cookie 同意コントロールを表示します。オフにすると、アプリケーションはこのコントロールがない場合と同じ動作になります。
admin-settings-key-cookies-mode = 同意モデル
admin-settings-help-cookies-mode = optIn: 訪問者が承諾するまで、任意のものは何も保存しません。optOut: 訪問者が拒否するまで、カテゴリのデフォルトが適用されます。noticeOnly: 訪問者に知らせるだけで、選択肢はありません。
admin-settings-key-cookies-region-mode = 厳格な地域での同意モデル
admin-settings-help-cookies-region-mode = 厳格な地域の訪問者に使うモデルです。same は、その地域でも上記の同意モデルを使います。
admin-settings-key-cookies-strict-regions = 厳格な地域
admin-settings-help-cookies-strict-regions = 厳格な地域の同意モデルを使う ISO 3166-1 の国コードです。デフォルトは EU と EEA の加盟国、英国、スイスです。
admin-settings-key-cookies-region-header = 国ヘッダー
admin-settings-help-cookies-region-header = プロキシまたは CDN が設定する、訪問者の国コードを保持するリクエストヘッダーの名前です。
admin-settings-key-cookies-unknown-region = 不明な国
admin-settings-help-cookies-unknown-region = 国を判定できないときに適用するモデルです。厳格な地域のモデルまたは通常のモデルから選びます。
admin-settings-key-cookies-functional-enabled = 機能性 Cookie を提供
admin-settings-help-cookies-functional-enabled = 同意コントロールに、設定と機能のカテゴリ（テーマ、言語、レイアウトの記憶）を表示します。
admin-settings-key-cookies-functional-default = 機能性のデフォルト
admin-settings-help-cookies-functional-default = 同意モデルがデフォルトを許可する場合の、機能性カテゴリの初期状態です。
admin-settings-key-cookies-analytics-enabled = 分析 Cookie を提供
admin-settings-help-cookies-analytics-enabled = 同意コントロールに分析カテゴリを表示します。
admin-settings-key-cookies-analytics-default = 分析のデフォルト
admin-settings-help-cookies-analytics-default = 同意モデルがデフォルトを許可する場合の、分析カテゴリの初期状態です。
admin-settings-key-cookies-marketing-enabled = マーケティング Cookie を提供
admin-settings-help-cookies-marketing-enabled = 同意コントロールにマーケティングカテゴリを表示します。
admin-settings-key-cookies-marketing-default = マーケティングのデフォルト
admin-settings-help-cookies-marketing-default = 同意モデルがデフォルトを許可する場合の、マーケティングカテゴリの初期状態です。
admin-settings-key-cookies-layout = レイアウト
admin-settings-help-cookies-layout = banner はページ全体にわたるバー、card は浮かぶカード、modal は訪問者が選択するまでページを操作できなくするダイアログです。
admin-settings-key-cookies-position = 位置
admin-settings-help-cookies-position = 同意コントロールをページ上のどこに表示するかです。
admin-settings-key-cookies-reject-prominent = 拒否を承諾と同じ目立ち方にする
admin-settings-help-cookies-reject-prominent = 「すべて拒否」を、「すべて承諾」と同じ大きさとスタイルで表示します。
admin-settings-key-cookies-policy-url = Cookie ポリシーの URL
admin-settings-help-cookies-policy-url = 同意コントロールに表示する、Cookie ポリシーへのリンクです。
admin-settings-key-cookies-privacy-url = プライバシーポリシーの URL
admin-settings-help-cookies-privacy-url = 同意コントロールに表示する、プライバシーポリシーへのリンクです。
admin-settings-key-cookies-policy-version = ポリシーのバージョン
admin-settings-help-cookies-policy-version = Cookie の使い方が変わったときにこの数字を上げると、訪問者に再度確認します。
admin-settings-key-cookies-lifetime-days = 同意の有効期間
admin-settings-help-cookies-lifetime-days = 訪問者に再度確認するまで、選択が有効な日数です。
admin-settings-key-cookies-reprompt-on-version = 新しいポリシーのバージョンで再度確認
admin-settings-help-cookies-reprompt-on-version = ポリシーのバージョンが、訪問者が選択したときのものより高い場合に、再度確認します。
admin-settings-key-cookies-honour-gpc = Global Privacy Control を尊重
admin-settings-help-cookies-honour-gpc = ブラウザーからの Global Privacy Control のシグナルを、分析とマーケティングの拒否として扱います。
admin-settings-key-cookies-honour-dnt = Do Not Track を尊重
admin-settings-help-cookies-honour-dnt = ブラウザーからの Do Not Track のシグナルを、分析とマーケティングの拒否として扱います。
admin-settings-key-cookies-log-consent = 同意ログを保持
admin-settings-help-cookies-log-consent = 統計とご自身の記録のために、各同意の選択をサーバー側に記録します。
admin-settings-key-cookies-log-retention-days = 同意ログの保持期間
admin-settings-help-cookies-log-retention-days = 同意ログの行が削除されるまでの保持日数です。
admin-settings-key-cookies-scan-interval-hours = ストレージのスキャン間隔
admin-settings-help-cookies-scan-interval-hours = ブラウザーの Cookie とストレージを自動スキャンする間隔（時間）です。0 の場合は、要求されたときのみスキャンします。
admin-settings-key-cookies-unclassified = 未分類のストレージ
admin-settings-help-cookies-unclassified = report は、どれにも分類されないストレージをそのままにして、確認用に一覧表示します。それ以外の値は、分類するまでそのカテゴリとして扱います。
admin-settings-key-cookies-text-title = 同意コントロールのタイトル
admin-settings-help-cookies-text-title = 言語ごとの同意コントロールのタイトルです。空の場合は、組み込みのテキストを使います。
admin-settings-key-cookies-text-body = 同意コントロールのテキスト
admin-settings-help-cookies-text-body = 言語ごとの同意コントロールの説明文です。空の場合は、組み込みのテキストを使います。
admin-settings-key-cookies-label-necessary = 必須カテゴリの名前
admin-settings-help-cookies-label-necessary = 言語ごとの必須カテゴリの名前です。空の場合は、組み込みの名前を使います。
admin-settings-key-cookies-description-necessary = 必須カテゴリの説明
admin-settings-help-cookies-description-necessary = 言語ごとの必須カテゴリの説明です。空の場合は、組み込みの説明を使います。
admin-settings-key-cookies-label-functional = 機能性カテゴリの名前
admin-settings-help-cookies-label-functional = 言語ごとの機能性カテゴリの名前です。空の場合は、組み込みの名前を使います。
admin-settings-key-cookies-description-functional = 機能性カテゴリの説明
admin-settings-help-cookies-description-functional = 言語ごとの機能性カテゴリの説明です。空の場合は、組み込みの説明を使います。
admin-settings-key-cookies-label-analytics = 分析カテゴリの名前
admin-settings-help-cookies-label-analytics = 言語ごとの分析カテゴリの名前です。空の場合は、組み込みの名前を使います。
admin-settings-key-cookies-description-analytics = 分析カテゴリの説明
admin-settings-help-cookies-description-analytics = 言語ごとの分析カテゴリの説明です。空の場合は、組み込みの説明を使います。
admin-settings-key-cookies-label-marketing = マーケティングカテゴリの名前
admin-settings-help-cookies-label-marketing = 言語ごとのマーケティングカテゴリの名前です。空の場合は、組み込みの名前を使います。
admin-settings-key-cookies-description-marketing = マーケティングカテゴリの説明
admin-settings-help-cookies-description-marketing = 言語ごとのマーケティングカテゴリの説明です。空の場合は、組み込みの説明を使います。
admin-settings-key-encryption-rotation-days = データベースキーのローテーション間隔（日）
admin-settings-help-encryption-rotation-days = 最後のキー変更から何日後に、データベースキーをローテーション期限として報告するかです。0 の場合は期限を設けません。キー自体が設定に保存されることはありません。
admin-settings-key-encryption-warnings-enabled = 暗号化に関する検出を報告
admin-settings-help-encryption-warnings-enabled = 平文のストアや、期限を過ぎたキーのローテーションを、デプロイ評価の検出項目として表示します。
admin-settings-key-retention-sweeper-enabled = 保持スイーパー
admin-settings-help-retention-sweeper-enabled = このページの上限より古い行を削除する日次スイープを実行します。オフにすると、再びオンにするまですべての行が保持されます。
admin-settings-key-retention-sweeper-batch-rows = バッチあたりの行数
admin-settings-help-retention-sweeper-batch-rows = 1 回の削除文で削除する行数です。バッチが小さいほど、データベースのロックを保持する時間が短くなります。
admin-settings-key-retention-sweeper-max-batches = 1 回の実行でストアごとのバッチ数
admin-settings-help-retention-sweeper-max-batches = 1 回の実行で、1 つのストアが実行できる削除文の最大数です。古い行がさらに多いストアは、次の実行で続きを処理します。
admin-settings-key-retention-sweeper-jitter-minutes = 開始時刻のばらつき（分）
admin-settings-help-retention-sweeper-jitter-minutes = 次の実行は、24 時間後からこの値までのランダムな分数だけ遅れて開始され、複数のサーバーが同時にスイープしないようにします。
admin-settings-key-retention-sweeper-vacuum-after-rows = 領域を回収する行数
admin-settings-help-retention-sweeper-vacuum-after-rows = 1 回の実行でこの行数以上を削除すると、データベースを圧縮(VACUUM)します。0 の場合、スイープ後に圧縮は行いません。週次のバキュームジョブは別です。
admin-settings-key-retention-password-change-audit-days = パスワード変更の記録（日）
admin-settings-help-retention-password-change-audit-days = パスワード変更の記録（アカウント、方法、結果）を保持する日数です。0 の場合は永久に保持します。
admin-settings-key-retention-sso-login-audit-days = シングルサインオンのログイン記録（日）
admin-settings-help-retention-sso-login-audit-days = シングルサインオンのログイン記録（プロバイダー、ハッシュ化されたサブジェクト、結果）を保持する日数です。0 の場合は永久に保持します。
admin-settings-key-retention-dlp-audit-days = データ損失防止の記録（日）
admin-settings-help-retention-dlp-audit-days = データ損失防止ルールに一致した記録を保持する日数です。0 の場合は永久に保持します。
admin-settings-key-retention-assist-audit-days = アシスタントのアクセス記録（日）
admin-settings-help-retention-assist-audit-days = アシスタントが機能を使った記録（実行者、機能、スコープの概要）を保持する日数です。0 の場合は永久に保持します。
admin-settings-key-retention-alert-events-days = 発生したアラートの記録（日）
admin-settings-help-retention-alert-events-days = 発生した管理者アラートの記録を保持する日数です。0 の場合は永久に保持します。
admin-settings-key-retention-job-runs-days = バックグラウンドジョブの履歴（日）
admin-settings-help-retention-job-runs-days = 完了したバックグラウンドジョブの実行記録を保持する日数です。実行中のものは削除されません。0 の場合は履歴を永久に保持します。
admin-settings-key-retention-outbox-days = 送信済みおよびキャンセル済みの送信記録（日）
admin-settings-help-retention-outbox-days = 配信済みまたはキャンセル済みメッセージの送信記録を保持する日数です。待機中、再試行中、失敗、または結果が不確かなメッセージは削除されません。0 の場合は永久に保持します。
admin-settings-key-retention-expired-credentials-grace-days = 期限切れの認証情報（期限後の日数）
admin-settings-help-retention-expired-credentials-grace-days = 期限切れになった、記憶済みデバイス、サインインのチャレンジ、サインイントークン、OAuth トークン、webhook のナンス、接続元の失敗カウンター、画像リンクのハンドルを、期限後に保持する日数です。0 の場合は次の実行で削除します。
admin-settings-key-retention-ip-addresses-days = 保存済み IP アドレスを縮約するまでの日数
admin-settings-help-retention-ip-addresses-days = この日数が経過すると、認証履歴と監査エントリの IP アドレスが切り詰められるかハッシュ化され、行は保持されます。0 の場合は、アドレスを記録されたまま保持します。
admin-settings-key-retention-ip-addresses-mode = IP 縮約の方法
admin-settings-help-retention-ip-addresses-mode = Truncate は IPv4 の /24 または IPv6 の /48 のプレフィックスを残します。Hash はアドレスを短いキー付きハッシュに置き換え、同じアドレスをグループ化できる状態を保ちます。
admin-settings-key-retention-user-enabled = ユーザーによる保持ルールの設定を許可
admin-settings-help-retention-user-enabled = ユーザーが、フォルダーやメールに対する自分の保持ルールを設定できます。オフにすると、既存のユーザールールの実行が停止します。下の組織全体のルールは引き続き実行されます。
admin-settings-key-retention-user-min-days = ユーザールールがメールを削除するまでの最小日数
admin-settings-help-retention-user-min-days = メールを削除するユーザールールは、この日数より新しいメールには作用できません。これより小さい値はルール保存時に拒否され、実行時には引き上げられます。
admin-settings-key-retention-user-max-days = ユーザールールがメールの削除を待てる最大日数
admin-settings-help-retention-user-max-days = メールを削除するユーザールールは、この日数より長く待つことはできません。0 の場合は上限を設けません。これより大きい値はルール保存時に拒否され、実行時には引き下げられます。
admin-settings-key-retention-user-max-rules = ユーザーあたりの保持ルールの最大数
admin-settings-help-retention-user-max-rules = 1 人のユーザーが保持できる保持ルールの最大数です。
admin-settings-key-retention-user-allow-permanent-delete = ユーザールールによるメールの完全削除を許可
admin-settings-help-retention-user-allow-permanent-delete = オフの場合、削除ルールは常にメールをゴミ箱へ移動し、完全削除の前に取り消しの猶予が適用されます。
admin-settings-key-retention-user-undo-days = 保持ルールの取り消し猶予（日）
admin-settings-help-retention-user-undo-days = 保持ルールでゴミ箱に移動されたメールが、完全削除されるまでそこに残る日数です。ユーザーはより短い期間を選べます。0 の場合は次の実行で完全削除します。
admin-settings-key-retention-user-mandatory = 組織全体のメールルール
admin-settings-help-retention-user-mandatory = すべてのアカウントに適用されるルールで、folder-role:days の形式のエントリです。例： trash:30 や junk:30。そのフォルダー内で指定日数より古いメールは完全削除されます。ユーザーはこれらを変更、一時停止、またはメールを除外することはできません。法的保全の対象のアカウントのメールはスキップされます。
admin-settings-key-retention-user-protected-folders = 保持ルールが対象にしないフォルダー
admin-settings-help-retention-user-protected-folders = どのユーザールールも対象にしないフォルダーで、ロール（drafts、sent、inbox、archive）またはフォルダー名で指定します。同じフォルダーについては、保護されたフォルダーが組織全体のルールより優先されます。
admin-settings-key-retention-user-templates = ユーザーに提供する保持テンプレート
admin-settings-help-retention-user-templates = ユーザーが設定で採用できるテンプレートを ID で指定します： newsletters30、receipts365、notifications14、trash30、junk30、read-archive180。
admin-settings-key-retention-user-run-for-restricted = 制限付きアカウントにもユーザールールを実行
admin-settings-help-retention-user-run-for-restricted = 処理の制限を受けているアカウントでも、ユーザーの保持ルールを実行するかどうかです。オフにすると、それらのアカウントはスキップされます。
admin-settings-key-retention-user-interval-hours = ユーザー保持ジョブの実行間隔（時間）
admin-settings-help-retention-user-interval-hours = 1 つのアカウントに対する、ユーザー保持ジョブの定期実行の間隔（時間）です。
admin-settings-key-retention-user-batch-messages = 保持処理 1 回あたりのメッセージ数
admin-settings-help-retention-user-batch-messages = 1 回の移動または削除の呼び出しで処理するメッセージ数です。最大 200 件まで使われます。
admin-settings-key-retention-user-max-per-run = 1 回の実行でアカウントごとに変更するメッセージの最大数
admin-settings-help-retention-user-max-per-run = ユーザー保持ジョブが、1 回の実行で 1 つのアカウントについて変更するメッセージの最大数です。残りは次の実行まで待ちます。
admin-settings-key-retention-user-pause-ms = 保持バッチの間の一時停止（ミリ秒）
admin-settings-help-retention-user-pause-ms = バッチ間の一時停止時間（ミリ秒）で、大規模なクリーンアップでメールサーバーに負荷がかからないようにします。
admin-settings-key-retention-class-short-days = 短期クラスの上限（日）
admin-settings-help-retention-class-short-days = 運用データ（セッション、トークン、レート制限、ジョブ履歴、メトリクス）の上限です。クラス制限がオンのときに適用されます。
admin-settings-key-retention-class-medium-days = 中期クラスの上限（日）
admin-settings-help-retention-class-medium-days = サインイン履歴、リクエスト保護の履歴、セキュリティ通知の上限です。短期クラスを下回ってはならず、それより小さい値は短期クラスの値に引き上げられます。
admin-settings-key-retention-class-long-days = 長期クラスの保持期間（日）
admin-settings-help-retention-class-long-days = 長期クラス（セキュリティ監査、管理者の操作、リクエスト、同意、削除の記録）を、アーカイブを含めて保持する日数です。0 の場合は永久に保持し、コンソールが警告を表示します。中期クラスを下回ってはなりません。
admin-settings-key-retention-class-enforce = 各ストアをクラスの上限に従わせる
admin-settings-help-retention-class-enforce = オンの場合、各ストア自身の上限がクラスの上限に収まるよう制限され、0（保持）はクラスの上限として扱われます。オフの場合は、ストアごとの上限がそのまま適用されます。
admin-settings-key-retention-archive-enabled = 長期クラスの古い行をアーカイブ
admin-settings-help-retention-archive-enabled = 長期クラスの古い行を、削除せずに、封印された圧縮アーカイブファイルへ移動します。

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-retention-archive-directory = アーカイブディレクトリ
admin-settings-help-retention-archive-directory = アーカイブファイルを置くディレクトリで、絶対パス、またはサーバーの作業ディレクトリからの相対パスです。親ディレクトリのセグメント(..)は拒否されます。
admin-settings-key-retention-archive-batch-rows = アーカイブファイルあたりの行数
admin-settings-help-retention-archive-batch-rows = 1 つのアーカイブファイルに書き込む行数です。
admin-settings-key-retention-archive-max-segments = 1 回の実行でストアごとのアーカイブファイル数
admin-settings-help-retention-archive-max-segments = 1 回の実行で、各ストアに対して書き込めるアーカイブファイルの数です。残りは次の実行まで待ちます。
admin-settings-key-retention-archive-key-generation = アーカイブ封印キーの世代
admin-settings-help-retention-archive-key-generation = 新しいアーカイブファイルを封印するキーの世代です。値を上げるとキーがローテーションされます。以前のファイルは、それぞれの世代で引き続き検証できます。
admin-settings-key-retention-archive-verify-days = アーカイブの確認間隔（日）
admin-settings-help-retention-archive-verify-days = アーカイブファイルの整合性チェックの間隔（日）です。
admin-settings-key-retention-archive-legal-hold = アーカイブの法的保全
admin-settings-help-retention-archive-legal-hold = 長期クラスの上限にかかわらず、すべてのアーカイブファイルを保持します。
admin-settings-key-logging-file-enabled = ログファイルを書き込む
admin-settings-help-logging-file-enabled = サーバーログをローテーションするファイルにも書き込みます。オフの場合は標準出力のみに出力し、これはコンテナーでの通常の使い方です。
admin-settings-key-logging-file-directory = ログディレクトリ
admin-settings-help-logging-file-directory = ログファイルを置くディレクトリで、絶対パス、またはサーバーの作業ディレクトリからの相対パスです。親ディレクトリのセグメント(..)は拒否されます。
admin-settings-key-logging-file-max-size-mb = サイズでローテーション(MB)
admin-settings-help-logging-file-max-size-mb = 現在のログファイルがこのサイズに達すると、閉じて新しいファイルを開始します。
admin-settings-key-logging-file-rotate-hours = 経過時間でローテーション（時間）
admin-settings-help-logging-file-rotate-hours = 現在のログファイルが、この時間を経過するとローテーションされます。0 の場合はサイズのみでローテーションします。
admin-settings-key-logging-file-max-age-days = ローテーション済みファイルの保持期間（日）
admin-settings-help-logging-file-max-age-days = これより古いローテーション済みログファイルは削除されます。
admin-settings-key-logging-file-max-files = ローテーション済みファイルの最大数
admin-settings-help-logging-file-max-files = この数を超えた分の、最も古いローテーション済みファイルが削除されます。
admin-settings-key-logging-file-max-total-mb = 合計サイズの上限(MB)
admin-settings-help-logging-file-max-total-mb = ログディレクトリがこのサイズを下回るまで、最も古いローテーション済みファイルが削除されます。
admin-settings-key-logging-file-compress = ローテーション済みファイルを圧縮
admin-settings-help-logging-file-compress = ローテーション済みのログファイルを gzip で圧縮します。
admin-settings-key-logging-level = ログレベル
admin-settings-help-logging-level = 下にある、独自のレベルを持たないコンポーネントに使うレベルです。この設定が読み込まれるまでは、起動時に MW_LOG 環境変数がレベルを決めます。
admin-settings-key-logging-component-levels = コンポーネントごとのログレベル
admin-settings-help-logging-component-levels = 1 行に 1 つの component=level のエントリを指定します。例： mw_imap=debug。レベルは error、warn、info、debug、trace です。
admin-settings-key-logging-redaction-addresses = ログ内のメールアドレス
admin-settings-help-logging-redaction-addresses = Full はアドレスをそのまま記録します。Truncated は最初の 1 文字とドメインを残します。Hashed はローカル部を短いハッシュに置き換えます。None はアドレスを削除します。
admin-settings-key-logging-redaction-ips = ログ内の IP アドレス
admin-settings-help-logging-redaction-ips = Full はアドレスをそのまま記録します。Truncated は IPv4 の /24 または IPv6 の /48 のプレフィックスを残します。Hashed はアドレスを短いハッシュに置き換えます。None は削除します。
admin-settings-key-logging-redaction-tokens = ログ内の機密情報
admin-settings-help-logging-redaction-tokens = ベアラートークン、API キー、URL 内のパスワードなどの機密情報を、ログ行から取り除きます。
admin-settings-key-retention-sessions-idle-days = 未使用のセッション（日）
admin-settings-help-retention-sessions-idle-days = アドレスとユーザーエージェントを含む、ブラウザーまたはネイティブクライアントのセッション記録が、削除されるまでに未使用でいられる日数です。セッションのタイムアウトが先に適用され、この設定は残った行を削除するだけです。0 の場合は保持します。
admin-settings-key-policy-public-pages = 公開ポリシーページ
admin-settings-help-policy-public-pages = 生成されたプライバシーポリシーを /privacy で、Cookie ポリシーを /cookies-policy で、サインインなしで提供します。
admin-settings-key-policy-auto-version = ポリシーバージョンの自動作成
admin-settings-help-policy-auto-version = ポリシーに記載されている設定が変更されたときに、新しいポリシーバージョンを記録します。オフの場合： 変更は「公開」まで保留されます。
admin-settings-key-policy-reconsent-on-change = 新しいポリシーバージョンで Cookie の同意を再度求める
admin-settings-help-policy-reconsent-on-change = 新しいポリシーバージョンが記録されたときに Cookie ポリシーのバージョンを上げ、訪問者に再度確認します（cookies.repromptOnVersion が必要です）。
admin-settings-key-policy-controller-name = 管理者の名称
admin-settings-help-policy-controller-name = データを処理する理由と方法を決定する組織の名称で、プライバシーポリシーに表示されます。
admin-settings-key-policy-controller-address = 管理者の住所
admin-settings-help-policy-controller-address = 管理者の郵送先住所で、プライバシーポリシーに表示されます。
admin-settings-key-policy-legal-bases = 法的根拠
admin-settings-help-policy-legal-bases = 依拠する法的根拠を、ご自身の言葉で記載します。プライバシーポリシーに表示されます。
admin-settings-key-policy-hosting-provider = ホスティングプロバイダー
admin-settings-help-policy-hosting-provider = データを誰がどこでホストしているかで、プライバシーポリシーに表示されます。
admin-settings-key-policy-hosting-transfers = ホスティング国外への移転
admin-settings-help-policy-hosting-transfers = 他国へのデータ移転と、採用している保護措置です。プライバシーポリシーに表示されます。
admin-settings-key-policy-supervisory-authority = 監督機関
admin-settings-help-policy-supervisory-authority = ユーザーが苦情を申し立てられる機関で、プライバシーポリシーに表示されます。
admin-settings-key-dpia-suggest = 影響評価を提案
admin-settings-help-dpia-suggest = アシスタント、開封トラッキング、分析が有効で、評価が存在しない場合に、推奨される影響評価を一覧表示します。
admin-settings-key-dpia-review-months = 影響評価の見直し間隔
admin-settings-help-dpia-review-months = 評価が承認されたときに設定される、見直し日までの月数です。
admin-settings-key-dpia-reminder-days = 影響評価のリマインダー期間
admin-settings-help-dpia-reminder-days = 見直し日の何日前から、評価を期限が近いものとして一覧表示するかです。
admin-settings-key-restriction-enabled = 処理の制限
admin-settings-help-restriction-enabled = 管理者または承認済みのリクエストによって、個人のデータの処理を制限できるかどうかです。
admin-settings-key-restriction-auto-apply-on-approval = リクエスト承認時に制限する
admin-settings-help-restriction-auto-apply-on-approval = 制限のリクエストが承認されると、すぐに制限を適用します。オフの場合： 管理者が「コンプライアンス > 制限」から適用します。
admin-settings-key-restriction-mail-delivery = 制限中の新着メール
admin-settings-help-restriction-mail-delivery = 制限の適用中に届いたメールについて、管理者が選んだ扱いです。保持、キューに保留、拒否のいずれかです。このサーバーはメールを受信しません。値は記録および表示され、メールサーバーがそれに従って動作します。
admin-settings-key-restriction-default-expiry-days = デフォルトの制限期間（日）
admin-settings-help-restriction-default-expiry-days = 新しい制限が自動的に終了するまでの日数です。0 の場合は、誰かが解除するまで続きます。
admin-settings-key-restriction-notify-user = 本人に通知
admin-settings-help-restriction-notify-user = 制限が適用または解除されたときに、本人に通知を送信します。
admin-settings-key-restriction-scope-indexing = 検索インデックスと埋め込みを一時停止
admin-settings-help-restriction-scope-indexing = 新しい制限で、検索インデックスと埋め込みをデフォルトで一時停止するかどうかです。
admin-settings-key-restriction-scope-assistant = アシスタントを一時停止
admin-settings-help-restriction-scope-assistant = 新しい制限で、アシスタントをデフォルトで一時停止するかどうかです。
admin-settings-key-restriction-scope-analytics = 分析と開封トラッキングを一時停止
admin-settings-help-restriction-scope-analytics = 新しい制限で、分析と開封トラッキングをデフォルトで一時停止するかどうかです。
admin-settings-key-restriction-scope-classification = ルールと自動振り分けを一時停止
admin-settings-help-restriction-scope-classification = 新しい制限で、ルールと自動振り分けをデフォルトで一時停止するかどうかです。
admin-settings-key-restriction-scope-push = プッシュ通知を一時停止
admin-settings-help-restriction-scope-push = 新しい制限で、プッシュ通知をデフォルトで一時停止するかどうかです。
admin-settings-key-restriction-scope-third-party = 接続済みのサービスとプラグインを一時停止
admin-settings-help-restriction-scope-third-party = 新しい制限で、接続済みのサービスとプラグインをデフォルトで一時停止するかどうかです。
admin-settings-key-consent-enabled = 目的ごとの同意
admin-settings-help-consent-enabled = ユーザーに目的と同意の選択肢を表示し、実行時に適用します。
admin-settings-key-consent-receipts-retention-days = 同意レシートの保持期間（日）
admin-settings-help-consent-receipts-retention-days = 同意レシートを保持する日数です。0 の場合は、本人が消去されるまで保持します。
admin-settings-key-consent-art22-notice = 自動処理に関する通知
admin-settings-help-consent-art22-notice = アシスタントと検索機能について、平易な言葉で書かれた通知と、機能ごとのスイッチをユーザーに表示します。
admin-settings-key-consent-analytics-basis = 法的根拠： 利用状況のカウント
admin-settings-help-consent-analytics-basis = ユーザーに表示する、利用状況のカウントの法的根拠です。「同意」の場合のみ、機能は承諾を待ちます。それ以外の根拠は、本人が撤回しない限り適用されます。どの根拠が適切かは管理者が決定します。
admin-settings-key-consent-analytics-version = テキストのバージョン： 利用状況のカウント
admin-settings-help-consent-analytics-version = 文言や処理が変わったときにこの数字を上げます。以前のバージョンに同意した人には、再度確認します。
admin-settings-key-consent-analytics-text = 独自の文言： 利用状況のカウント
admin-settings-help-consent-analytics-text = この目的に対する、言語ごとの独自のテキストです。空の場合は、組み込みの文言を使います。
admin-settings-key-consent-analytics-tag-basis = 法的根拠： 分析プロバイダーのスクリプト
admin-settings-help-consent-analytics-tag-basis = ユーザーに表示する、分析プロバイダーのスクリプトの法的根拠です。「同意」の場合のみ、機能は承諾を待ちます。それ以外の根拠は、本人が撤回しない限り適用されます。どの根拠が適切かは管理者が決定します。
admin-settings-key-consent-analytics-tag-version = テキストのバージョン： 分析プロバイダーのスクリプト
admin-settings-help-consent-analytics-tag-version = 文言や処理が変わったときにこの数字を上げます。以前のバージョンに同意した人には、再度確認します。
admin-settings-key-consent-analytics-tag-text = 独自の文言： 分析プロバイダーのスクリプト
admin-settings-help-consent-analytics-tag-text = この目的に対する、言語ごとの独自のテキストです。空の場合は、組み込みの文言を使います。
admin-settings-key-consent-assistant-basis = 法的根拠： アシスタント
admin-settings-help-consent-assistant-basis = ユーザーに表示する、アシスタントの法的根拠です。「同意」の場合のみ、機能は承諾を待ちます。それ以外の根拠は、本人が撤回しない限り適用されます。どの根拠が適切かは管理者が決定します。
admin-settings-key-consent-assistant-version = テキストのバージョン： アシスタント
admin-settings-help-consent-assistant-version = 文言や処理が変わったときにこの数字を上げます。以前のバージョンに同意した人には、再度確認します。
admin-settings-key-consent-assistant-text = 独自の文言： アシスタント
admin-settings-help-consent-assistant-text = この目的に対する、言語ごとの独自のテキストです。空の場合は、組み込みの文言を使います。
admin-settings-key-consent-push-basis = 法的根拠： プッシュ通知
admin-settings-help-consent-push-basis = ユーザーに表示する、プッシュ通知の法的根拠です。「同意」の場合のみ、機能は承諾を待ちます。それ以外の根拠は、本人が撤回しない限り適用されます。どの根拠が適切かは管理者が決定します。
admin-settings-key-consent-push-version = テキストのバージョン： プッシュ通知
admin-settings-help-consent-push-version = 文言や処理が変わったときにこの数字を上げます。以前のバージョンに同意した人には、再度確認します。
admin-settings-key-consent-push-text = 独自の文言： プッシュ通知
admin-settings-help-consent-push-text = この目的に対する、言語ごとの独自のテキストです。空の場合は、組み込みの文言を使います。
admin-settings-key-consent-open-tracking-basis = 法的根拠： 開封トラッキング
admin-settings-help-consent-open-tracking-basis = ユーザーに表示する、開封トラッキングの法的根拠です。「同意」の場合のみ、機能は承諾を待ちます。それ以外の根拠は、本人が撤回しない限り適用されます。どの根拠が適切かは管理者が決定します。
admin-settings-key-consent-open-tracking-version = テキストのバージョン： 開封トラッキング
admin-settings-help-consent-open-tracking-version = 文言や処理が変わったときにこの数字を上げます。以前のバージョンに同意した人には、再度確認します。
admin-settings-key-consent-open-tracking-text = 独自の文言： 開封トラッキング
admin-settings-help-consent-open-tracking-text = この目的に対する、言語ごとの独自のテキストです。空の場合は、組み込みの文言を使います。
admin-settings-key-consent-remote-content-basis = 法的根拠： リモート画像
admin-settings-help-consent-remote-content-basis = ユーザーに表示する、リモート画像の法的根拠です。「同意」の場合のみ、機能は承諾を待ちます。それ以外の根拠は、本人が撤回しない限り適用されます。どの根拠が適切かは管理者が決定します。
admin-settings-key-consent-remote-content-version = テキストのバージョン： リモート画像

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-consent-remote-content-version = 文言や処理が変わったときにこの数字を上げます。以前のバージョンに同意した人には、再度確認します。
admin-settings-key-consent-remote-content-text = 独自の文言： リモート画像
admin-settings-help-consent-remote-content-text = この目的に対する、言語ごとの独自のテキストです。空の場合は、組み込みの文言を使います。
admin-settings-key-consent-integrations-basis = 法的根拠： 接続済みのサービス
admin-settings-help-consent-integrations-basis = ユーザーに表示する、接続済みのサービスの法的根拠です。「同意」の場合のみ、機能は承諾を待ちます。それ以外の根拠は、本人が撤回しない限り適用されます。どの根拠が適切かは管理者が決定します。
admin-settings-key-consent-integrations-version = テキストのバージョン： 接続済みのサービス
admin-settings-help-consent-integrations-version = 文言や処理が変わったときにこの数字を上げます。以前のバージョンに同意した人には、再度確認します。
admin-settings-key-consent-integrations-text = 独自の文言： 接続済みのサービス
admin-settings-help-consent-integrations-text = この目的に対する、言語ごとの独自のテキストです。空の場合は、組み込みの文言を使います。
admin-settings-key-consent-telemetry-basis = 法的根拠： 読み込み時間の測定
admin-settings-help-consent-telemetry-basis = ユーザーに表示する、読み込み時間の測定の法的根拠です。「同意」の場合のみ、機能は承諾を待ちます。それ以外の根拠は、本人が撤回しない限り適用されます。どの根拠が適切かは管理者が決定します。
admin-settings-key-consent-telemetry-version = テキストのバージョン： 読み込み時間の測定
admin-settings-help-consent-telemetry-version = 文言や処理が変わったときにこの数字を上げます。以前のバージョンに同意した人には、再度確認します。
admin-settings-key-consent-telemetry-text = 独自の文言： 読み込み時間の測定
admin-settings-help-consent-telemetry-text = この目的に対する、言語ごとの独自のテキストです。空の場合は、組み込みの文言を使います。
admin-settings-key-breach-enabled = 侵害台帳
admin-settings-help-breach-enabled = 台帳と通知期限のカウントをオンにします。
admin-settings-key-breach-notice-hours = 監督機関へ通知するまでの時間
admin-settings-help-breach-notice-hours = 認識してから監督機関へ通知するまでの時間です。通常は 72 時間です。
admin-settings-key-breach-reminders-hours = リマインダーの時刻
admin-settings-help-breach-reminders-hours = 期限の何時間前にリマインダーを出すかです。例： 6、24、48。
admin-settings-key-breach-reminders-overdue-repeat-hours = 期限超過後の繰り返し
admin-settings-help-breach-reminders-overdue-repeat-hours = 期限を過ぎた後の、リマインダーの間隔（時間）です。0 の場合は 1 回だけ通知します。
admin-settings-key-breach-authority-name = 監督機関
admin-settings-help-breach-authority-name = 監督機関への通知書に記載する名称です。
admin-settings-key-breach-authority-url = 監督機関の通知ページ
admin-settings-help-breach-authority-url = 監督機関の通知ページのアドレスです。
admin-settings-key-breach-authority-contact = 監督機関の連絡先
admin-settings-help-breach-authority-contact = 監督機関の連絡先です。
admin-settings-key-breach-dpo-name = データ保護の連絡担当者名
admin-settings-help-breach-dpo-name = 通知書で連絡窓口として記載する名前です。
admin-settings-key-breach-dpo-email = データ保護の連絡担当者のメール
admin-settings-help-breach-dpo-email = 通知書で連絡窓口として記載するアドレスです。
admin-settings-key-breach-dpo-phone = データ保護の連絡担当者の電話番号
admin-settings-help-breach-dpo-phone = 通知書に記載する電話番号です。
admin-settings-key-breach-template-subject = メッセージの件名
admin-settings-help-breach-template-subject = 影響を受けた人に送るメッセージの件名で、言語ごとに設定します。
admin-settings-key-breach-template-body = メッセージ本文
admin-settings-help-breach-template-body = 影響を受けた人に送るメッセージの本文で、言語ごとに設定します。
admin-settings-key-breach-notify-audit = 通知を監査ログに記録
admin-settings-help-breach-notify-audit = リマインダーと検出がセキュリティイベントになります。
admin-settings-key-breach-notify-webhook-urls = Webhook のアドレス
admin-settings-help-breach-notify-webhook-urls = リマインダーとメッセージを通知するアドレスです。機密として扱ってください。各呼び出しには署名が付きます。
admin-settings-key-breach-notify-operator-addresses = 運用担当者のメールボックス
admin-settings-help-breach-notify-operator-addresses = リマインダーを自分の受信トレイで受け取る、ローカルのメールボックスのアドレスです。
admin-settings-key-breach-subjects-in-app = 本人のアカウント内での通知
admin-settings-help-breach-subjects-in-app = 本人のアカウント内にメッセージを表示します。
admin-settings-key-breach-subjects-inbox = 本人のメールボックスへのメッセージ
admin-settings-help-breach-subjects-inbox = メッセージを本人自身のメールボックスに配信します。
admin-settings-key-breach-subjects-webhook = Webhook にも送信
admin-settings-help-breach-subjects-webhook = メッセージ本文を webhook のアドレスにも送信します。
admin-settings-key-breach-retention-years = 台帳の保持年数
admin-settings-help-breach-retention-years = 終了したインシデントを台帳に残す年数です。
admin-settings-key-breach-auto-enabled = 自動化
admin-settings-help-breach-auto-enabled = 検出をもとに、インシデントの下書きを作成します。
admin-settings-key-breach-auto-start-clock = 下書きのカウント開始
admin-settings-help-breach-auto-start-clock = 下書きの作成時にカウントを開始するか、確定したときのみ開始します。
admin-settings-key-breach-auto-assignee = 下書きの割り当て先
admin-settings-help-breach-auto-assignee = 下書きを誰に割り当てるかです。
admin-settings-key-breach-auto-escalate-after-hours = エスカレーションまでの時間
admin-settings-help-breach-auto-escalate-after-hours = 下書きがエスカレーションされるまでの待機時間（時間）です。0 の場合はエスカレーションしません。
admin-settings-key-breach-auto-escalate-to = エスカレーション先
admin-settings-help-breach-auto-escalate-to = エスカレーションされた下書きを誰に割り当てるかです。
admin-settings-key-breach-auto-window-minutes = カウントのウィンドウ（分）
admin-settings-help-breach-auto-window-minutes = ルールがイベントをカウントする期間（分）です。
admin-settings-key-breach-auto-theft-threshold = セッション窃取のアラート
admin-settings-help-breach-auto-theft-threshold = ウィンドウ内でこの件数のアラートがあると、下書きを作成します。0 の場合はオフです。
admin-settings-key-breach-auto-credential-stuffing-threshold = サインインのスプレーまたは急増
admin-settings-help-breach-auto-credential-stuffing-threshold = ウィンドウ内でこの件数のイベントがあると、下書きを作成します。0 の場合はオフです。
admin-settings-key-breach-auto-mass-forward-threshold = 大量の転送ルール
admin-settings-help-breach-auto-mass-forward-threshold = ウィンドウ内でこの件数のイベントがあると、下書きを作成します。0 の場合はオフです。
admin-settings-key-breach-auto-role-escalation-threshold = 管理者ロールの変更
admin-settings-help-breach-auto-role-escalation-threshold = ウィンドウ内でこの件数の変更があると、下書きを作成します。0 の場合はオフです。
admin-settings-key-breach-auto-export-volume-threshold = 通常と異なるエクスポート
admin-settings-help-breach-auto-export-volume-threshold = ウィンドウ内でこの件数のエクスポートがあると、下書きを作成します。0 の場合はオフです。
admin-settings-key-breach-auto-audit-chain-threshold = 監査チェーンの失敗
admin-settings-help-breach-auto-audit-chain-threshold = この件数の失敗があると、下書きを作成します。0 の場合はオフです。
admin-settings-key-breach-auto-backup-anomaly-threshold = バックアップまたは復元の失敗
admin-settings-help-breach-auto-backup-anomaly-threshold = ウィンドウ内でこの件数の失敗があると、下書きを作成します。0 の場合はオフです。
admin-settings-key-breach-auto-dsr-export-threshold = 多数のデータリクエストのエクスポート
admin-settings-help-breach-auto-dsr-export-threshold = ウィンドウ内でこの件数のエクスポートがあると、下書きを作成します。0 の場合はオフです。
admin-settings-key-ratelimit-queue-ms = 空きを待つ最長時間
admin-settings-help-ratelimit-queue-ms = リクエストが空き（JMAP の同時実行、画像デコーダー、プレビューワーカー）を待つ最長時間で、超えると 429 と Retry-After で拒否されます。範囲は 0-30000 ミリ秒、デフォルトは 3000 です。0 の場合はすぐに拒否します。
admin-settings-key-ratelimit-jmap-per-account-concurrent = アカウントあたりの同時 JMAP リクエスト数
admin-settings-help-ratelimit-jmap-per-account-concurrent = サインイン済みの 1 つのアカウントが、タブやデバイスをまたいで同時に実行できる JMAP リクエスト数です。他のアカウントや、同じアドレスの背後にいる別のユーザーと共有されることはありません。範囲は 4-64、デフォルトは 8 です。
admin-settings-key-ratelimit-jmap-global-concurrent = 全アカウントでの同時 JMAP リクエスト数
admin-settings-help-ratelimit-jmap-global-concurrent = サーバープロセス全体で、全アカウントを合わせて同時に実行する JMAP リクエスト数です。これを超えたリクエストはキュー時間のあいだ待ちます。範囲は 4-4096、デフォルトは 256 です。
admin-settings-key-ratelimit-session-burst = セッションあたりの連続タイミングレポート数
admin-settings-help-ratelimit-session-burst = 1 つのセッションが連続して送信できるクライアントのタイミングレポート数です。このエンドポイントはアドレスではなく、セッションごとに管理されます。範囲は 1-1000、デフォルトは 60 です。
admin-settings-key-ratelimit-session-per-minute = 1 分あたりに回復するタイミングレポート数
admin-settings-help-ratelimit-session-per-minute = バースト後に、セッションが 1 分あたりに回復するクライアントのタイミングレポート数です。範囲は 1-600、デフォルトは 30 です。
admin-settings-key-ratelimit-anon-burst = 連続する匿名の同意リクエスト数
admin-settings-help-ratelimit-anon-burst = 1 つのアドレスが、匿名の Cookie 同意ルートに連続して行えるリクエスト数です。1 つのアドレスの背後にあるオフィスを想定したサイズです。範囲は 1-1000、デフォルトは 60 です。
admin-settings-key-ratelimit-anon-per-minute = 1 分あたりに回復する匿名の同意リクエスト数
admin-settings-help-ratelimit-anon-per-minute = 匿名の Cookie 同意ルートで、アドレスが 1 分あたりに回復するリクエスト数です。範囲は 1-600、デフォルトは 30 です。
admin-settings-key-ratelimit-trusted-multiplier = 既知の接続元からのアカウント許容量(%)
admin-settings-help-ratelimit-trusted-multiplier = サインインが成功した記録のある接続元からリクエストが来たときに、サインイン済みのメールボックスが得られるアカウントのリクエスト許容量に対する割合です。100 で基本の許容量のままです。匿名、管理者、ユーザーエージェントの予算や、有効なブロックを緩和することはありません。範囲は 100-1000、デフォルトは 150 です。

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-attachments-bar-click-to-toggle = リーダーの添付ファイル バーのどこかをクリックすると、展開されるか折りたたまれます。オフの場合、矢印ボタンのみが機能します。

admin-settings-help-attachments-bar-download-all = 添付ファイル バーに、許可された添付ファイルを 1 つの ZIP ファイルとして保存する [すべてダウンロード] ボタンが表示されるかどうか。

admin-settings-help-attachments-bar-show-download-buttons = バー内の各添付ファイルにダウンロード ボタンが表示されるタイミング: ホバーまたはフォーカス時 (デフォルト)、常に、またはまったく表示されません。ダウンロードは引き続き添付ファイル ポリシーと mail.allowDownload に従います。

admin-settings-help-attachments-bar-zip-inline-images = メッセージテキスト内に表示される画像がすべてダウンロード ZIP に含まれるかどうか。デフォルトではオフです。

admin-settings-help-cap-reading-delete-sender = 確認後に一致するすべての送信者メッセージをゴミ箱に移動するリーダー アクションをユーザーが有効にできるようにします。デフォルトでは無効になっています。機能プロファイルでもそれを許可する必要があります。

admin-settings-help-compose-feature-family = 機能 ID によってコンポーザー コントロールのオンとオフを切り替えます。デフォルトはオンです。ユーザーが変更できないようにするには、ロックしてください。

admin-settings-help-compose-headers-allow-auto-submitted = ユーザーがメッセージに自動送信 (RFC 3834) を設定できるようにします。デフォルトではオフ: 他のシステムに応答しないように指示します。

admin-settings-help-compose-headers-allow-custom = ユーザーが独自のヘッダーを追加できるようにします (最大 20 個、名前はチェックされ、保護されたヘッダーは拒否されます)。

admin-settings-help-compose-headers-allow-return-receipt-to = ユーザーが従来の Return-Receipt-To ヘッダーを使用して領収書をリクエストできるようにします。デフォルトではオフです。

admin-settings-help-compose-headers-allow-threading-edit = ユーザーは、In-Reply-To と References を編集したり、返信から新しいスレッドを開始したりできます。

admin-settings-help-compose-headers-forced = 1 行に 1 つの Name=value ペアが、ドメインから送信されるすべてのメッセージに追加され、ユーザーの値を置き換えます。保護されたヘッダーは拒否されます。

admin-settings-help-compose-insert-image-resize-default = [ファイルからの画像] ダイアログの開始サイズ: オリジナル、大 (1600 ピクセル)、または中 (1024 ピクセル)。

admin-settings-help-compose-insert-max-inline-image-mb = これより大きい画像はメッセージ本文に配置されません。まだ取り付け可能です。 1から50まで。

admin-settings-help-compose-layout-default-font = 新しいメッセージのフォント ファミリ、サイズ、色は family|size|#colour で始まります。 Empty はテーマ フォントを使用します。

admin-settings-help-compose-reply-to-allow-override = オフの場合、アイデンティティの返信先がそのまま使用され、ユーザーは編集または削除できません。

admin-settings-help-compose-reply-to-show = 作成者が Reply-To 行を表示するかどうか: ID または元のメッセージに Reply-To 行がある場合のみ、常に表示する、または表示しない。

admin-settings-help-compose-sent-folder-fixed-id = ルールが「固定フォルダー」の場合に使用されるフォルダーの役割または ID。

admin-settings-help-compose-sent-folder-mode = 送信されたメッセージのコピーの保存場所: 通常の送信フォルダー、1 つの固定フォルダー、受信者ドメインごと、ID ごと、または件名ルールごと。

admin-settings-help-compose-sent-folder-per-identity = 1 行に 1 つの ID=フォルダー ペア。ルールが「ID による」場合に使用されます。

admin-settings-help-compose-sent-folder-rules = 1 行に 1 つのルール、domain:example.com= フォルダーまたは subject:text= フォルダー。最初の試合が勝ちます。

admin-settings-help-compose-sent-folder-skip-copy = 送信したメッセージのコピーは絶対に保管しないでください。管理者はこれを強制的にオンまたはオフにすることができます。

admin-settings-help-compose-styles-custom = ユーザー自身の作曲家スタイルは、スタイル ギャラリーによって作成された JSON となります。ユーザーが設定します。ここでは設定するものは何もありません。

admin-settings-help-compose-styles-default-set = 作曲家のスタイル ギャラリーの外観: デフォルト、モダン、クラシック、またはコンパクト。

admin-settings-help-compose-styles-max-custom = ユーザーがコンポーザーに保持できるほとんどのカスタム スタイル。 0 はスタイルの作成をオフにします。

admin-settings-help-contacts-save-from-message-add-note = メッセージから保存された連絡先に「<日付> のメッセージから保存されました」というメモを追加します。ユーザーとドメインの上書きはロックできます。

admin-settings-help-contacts-save-from-message-default-book = メッセージから保存された連絡先を受け取るアドレス帳 ID。 「default」はデフォルトのアドレス帳を使用します。ユーザーとドメインの上書きはロックできます。

admin-settings-help-contacts-save-from-message-default-scope = [連絡先の保存] ダイアログで最初にチェックされる参加者は、送信者のみ、送信者と返信先、または全員です。ユーザーとドメインの上書きはロックできます。

admin-settings-help-contacts-save-from-message-enabled = オファー このメッセージの連絡先をメッセージ メニューに保存します。オフの場合、項目は非表示になります。ユーザーとドメインの上書きはロックできます。

admin-settings-help-contacts-save-from-message-skip-automated = 手動でチェックを入れない限り、非返信アドレス、通知アドレス、およびバウンス アドレスを [連絡先の保存] ダイアログから除外したままにしておきます。ユーザーとドメインの上書きはロックできます。

admin-settings-help-contacts-save-sender-enabled = ワンクリックで送信者を連絡先アイテムとして保存する機能をメッセージ メニューに提供します。オフの場合、項目は非表示になります。ユーザーとドメインの上書きはロックできます。

admin-settings-help-contacts-save-sender-use-reply-to = 送信者がメーリング リストまたは通知アドレスである場合は、From ではなく Reply-To アドレスを保存します。ユーザーとドメインの上書きはロックできます。

admin-settings-help-dsr-admin-step-up = オン (デフォルト): データ要求の承認、消去の実行、または一括承認を行う管理者は、まず自分であることを再度確認する必要があります (パスワードと第 2 要素、またはパスキー)。確認には 5 分間かかります。オフにすると、余分なチェックが削除されます。決定は依然としてリクエスト証跡に記録されます。

admin-settings-help-export-default-format = エクスポートダイアログの開始形式。

admin-settings-help-export-default-scope = エクスポート ダイアログの開始範囲: メッセージ、会話、選択範囲、またはフォルダー。

admin-settings-help-export-dialog-enabled = メッセージ ビューでエクスポートすると、メッセージ メニューが適切な場所にダイアログを開きます。オフの場合、これらのメニューは 1 つのメッセージを .eml としてダウンロードします。

admin-settings-help-export-folder-enabled = ユーザーがフォルダー全体をメッセージの ZIP としてダウンロードできるようにします。各エクスポートは監査されます。

admin-settings-help-export-folder-format = フォルダー エクスポートが提供する形式: .eml ファイルの ZIP、mbox ファイルを保持する ZIP、またはその両方。

admin-settings-help-export-folder-include-subfolders = フォルダーのエクスポートにデフォルトでサブフォルダーが含まれるかどうか。これは、ZIP にフォルダー構造として保持されます。

admin-settings-help-export-folder-max-bytes = 1 つのフォルダーにエクスポートできるメッセージの最大バイト数。表示されているサイズより大きなフォルダーは拒否されます。

admin-settings-help-export-folder-max-messages = 1 つのフォルダーにエクスポートできるメッセージの最大数。

admin-settings-help-export-formats = エクスポート ダイアログが提供する形式: eml、mbox、txt、md、msg、docx、csv、json、pdf。サーバーは、リストされたサーバー構築フォーマットを拒否します。

admin-settings-help-export-message-download-enabled = ユーザーがメッセージを .eml、ヘッダーのみ、または mbox ファイルとしてダウンロードできるようにします。

admin-settings-help-folders-hide-empty = サイドバーの空のフォルダーを非表示にします。この設定がロックされていない限り、ユーザーはデフォルトを変更したり、個々のフォルダーに対してデフォルトを上書きしたりできます。メールは削除されません。

admin-settings-help-folders-pinned-hide-from-tree = ピン留めされたフォルダーは、フォルダー ツリーの上のピン留めされたグループに表示され、ツリー内の場所も維持されます。 [オン] にすると、サブフォルダーを持たない固定フォルダーがツリーから非表示になり、固定グループにのみ表示されます。サブフォルダーを含む固定フォルダーはツリー内に残ります。

admin-settings-help-mail-badge-auth = リーダーのヘッダーに DMARC 認証バッジが表示される場合: 常に、注目に値する場合のみ、またはまったく表示されません。

admin-settings-help-mail-badge-density = リーダーヘッダーのバッジのサイズ: コンパクトまたは快適。

admin-settings-help-mail-badge-folder = リーダーのヘッダーにフォルダー バッジが表示される場合: 常に、注目に値する場合のみ、またはまったく表示されません。

admin-settings-help-mail-badge-headers = メッセージの Expires、Reply-By、Sensitivity、および Supersedes ヘッダーのバッジをリーダーに表示します。

admin-settings-help-mail-badge-labels = リーダーのヘッダー バッジにテキスト付きのアイコンが表示されるか、アイコンのみが表示されるか。

admin-settings-help-mail-badge-list = リーダーのヘッダーにメーリング リストのバッジが表示される場合: 常に、注目に値する場合のみ、またはまったく表示されません。 List-Id または List-Unsubscribe ヘッダーが必要です。

admin-settings-help-mail-badge-sanitized = リーダーのヘッダーにサニタイズされた HTML バッジが表示される場合: 常に、注目に値する場合のみ、またはまったく表示されません。

admin-settings-help-mail-badge-spam = リーダーのヘッダーにスパム スコア バッジが表示される場合: 常に、注目に値する場合のみ、またはまったく表示されません。

admin-settings-help-mail-brand-avatars-require-auth = Never: ドメインが一致するたびにロゴを表示します。 known-failure (デフォルト): キャッシュされたメッセージの DKIM または DMARC の結果が失敗でない限り、それを表示します。チェックされていないメッセージはロゴを保持します。常に: ブランド独自のドメインの DKIM と調整された DMARC パスの後にのみ表示します。ユーザーとドメインの上書きはロックできます。

admin-settings-help-mail-compose-header-auto-collapse-seconds = 完全な受信者と件名のヘッダーがその概要に折りたたまれる前に、本文がフォーカスを保持しなければならない秒数。時間制限付きフォールディングを無効にする場合は 0、または 1 ～ 60 秒を選択します。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-mail-compose-priority-default = コンポーザー内で新しいメッセージが開始される優先順位。

admin-settings-help-mail-compose-priority-enabled = 「composer Options」グループに「Normal」、「High」、「Low」の優先度セレクターを表示します。オフの場合、新しいメールは優先ヘッダーなしで送信されます。

admin-settings-help-mail-expiry-hide-expired = 有効期限が過ぎたメッセージをどうするか: 何もしない、淡色表示する、またはアーカイブすることを提案します。

admin-settings-help-mail-list-keep = 送信者とメーリング リストは、sender:address または list:List-Id として 1 行に 1 つずつ常に保持されます。この送信者をクリーンアップし、購読を解除する提案と、保持ルールによってスキップされます。

admin-settings-help-mail-list-page-size = ユーザーが最後までスクロールするたびにリストが取得するメッセージの数。デフォルトは 50、20 ～ 200 です。これをロックしない限り、ユーザーは独自の値を選択します。

admin-settings-help-mail-list-unsubscribe-mode = リストでワンクリックリクエストが提供されている場合の購読解除の仕組み: 最初に質問するか、すぐに送信するか、リストのリンクまたは下書きのみを開きます。

admin-settings-help-mail-message-list-group = 受信または送信した日付、送信者、件名、重要性、サイズごとに会話をグループ化します。ロックされていない限り、ユーザーはこれをオーバーライドできます。

admin-settings-help-mail-priority-marks = メッセージ リスト、会話、リーダー ショーにマークを付ける優先度 (高と低の両方、高のみ、またはなし)。ロックされていない限り、ユーザーは [設定] > [フォルダとサイドバー] で選択します。

admin-settings-help-mail-priority-marks-style = 優先度マークの外観: アイコン、「高」または「低」という単語が表示されたアイコン、または細いバー。アイコンとテキストは色がなくても意味を保ちます。

admin-settings-help-mail-priority-send-headers = 優先度の高いメッセージまたは低いメッセージが送信時に取得されるヘッダー: Importance、X-Priority、および X-MSMail-Priority のいずれか。

admin-settings-help-mail-priority-trust-sender-headers = 送信者が設定した優先順位が表示されるかどうか: 常に、メッセージが整列された DMARC チェックに合格した場合のみ、またはまったく表示されません。すべてのメッセージに高優先度のマークを付ける送信者に対しては、trusted または Never を使用します。

admin-settings-help-mail-reply-from-header-priority = メッセージ ヘッダーは、応答の ID を選択するときにメッセージの送信先アドレスを見つけるためにこの順序で読み取られます。名前: Delivered-To、X-Original-To、Envelope-To、X-Forwarded-To、Resent-To、To、Cc。省略されたヘッダーは読み込まれません。

admin-settings-help-mail-search-default-scope = 現在のフォルダーまたはすべてのフォルダーで新しい通常のメール検索を開始します。これは、ロック時を含む初期デフォルトを制御します。ユーザーは、特定の検索に対して別の範囲を選択することもできます。保存され、明示的にスコープが設定されたクエリは保持されます。

admin-settings-help-metrics-dimensions-enabled = 保存された統計をクライアントの種類 (Web) とデバイス クラス (デスクトップ、モバイル) ごとに分割して保存します。オフにすると、メトリックおよび分ごとに 1 つの結合された行が保存されます。

admin-settings-help-metrics-persist-enabled = フラッシュ間隔ごと、およびサーバーの停止時にパフォーマンス統計 (イメージ プロキシ速度、ブラウザーの読み込み時間、レート制限およびリクエスト保護数) をデータベースに保存します。これにより、再起動後もグラフの履歴が保持されます。個人、住所、メッセージを特定するものは何もありません。メモリ内にのみ保存するにはオフにします。

admin-settings-help-metrics-persist-flush-seconds = パフォーマンス統計がデータベースに書き込まれる頻度 (5 ～ 3600 秒)。警告なしにサーバーが強制終了された場合、最大でもこれだけの損失が発生します。

admin-settings-help-observability-client-capture-console = また、console.error 呼び出しも、1 ページあたり最大 5 回ずつ報告します。ライブラリは予期される条件をログに記録するため、デフォルトではオフになっています。

admin-settings-help-observability-client-capture-csp = ブラウザーが検出した Content-Security-Policy 違反を、ブロックされたディレクティブとオリジンのみを使用して報告します。デフォルトではオンです。

admin-settings-help-observability-client-enabled = [オブザーバビリティ] > [クライアント エラー] に表示される、スクラブされたスタック トレースを使用して、Web アプリからエラー レポート (エラー境界、捕捉されなかったエラー、失敗したチャンク、ワーカー エラー、CSP 違反) を収集します。デフォルトではオンです。レポートには、メッセージの内容、アドレス、トークン、添付ファイル名、または URL クエリは含まれません。収集を停止するにはオフにします。既存のレポートは保持期間とともに期限切れになります。

admin-settings-help-observability-client-forward = また、スクラブされた各レポートを、MW_ERROR_FORWARD_URL で構成されたエラー シンクに送信します。デフォルトではオフです。シンクが構成されていない場合、何も送信されません。

admin-settings-help-observability-client-include-account-hash = サインイン セッションから送信されるレポートにアカウントのキー付き一方向ハッシュを追加すると、名前を付けなくても 1 つのアカウントからのレポートを区別できます。デフォルトではオフです。プライバシー キーがローテーションされると、ハッシュが変化します。

admin-settings-help-observability-client-max-per-session = ほとんどのエラー レポートは 1 ページの読み込みで送信される可能性があります (1 ～ 1000)。 60 秒以内に同じエラーが繰り返された場合は 1 回カウントされます。デフォルトは 20。

admin-settings-help-observability-client-sample-rate = ブラウザが送信するエラー レポートの割合 (0 ～ 1)。デフォルトは 1 (すべて)。各ページの読み込みは依然としてセッションあたりの制限で停止します。

admin-settings-help-observability-client-source-maps = ビルドが非表示のソース マップ (MW_SOURCEMAP=hidden でビルド) を出荷したときに、スタック トレース内のバンドルの位置をソース ファイルに解決します。マップはサーバーによってのみ読み取られ、ブラウザーには決して提供されません。デフォルトではオンです。マップがなければ生のフレームが表示されます。

admin-settings-help-privacy-error-reports = アプリの一部が失敗したときにブラウザがエラー レポートを送信するかどうか。デフォルトではオンです。グローバル、ドメイン、およびユーザーのスコープ: ここでデフォルトを設定するか、ロックします。ロックされていない限り、[設定] > [プライバシー] で自分でオフにできます。

admin-settings-help-privacy-image-cache-max-stale-secs = キャッシュされた画像は、その鮮度を数秒経過しても、オリジンに到達できない場合に応答し続けます。再検証ウィンドウを下回らないようにしてください。範囲は 0 ～ 604800。デフォルトは86400です。

admin-settings-help-privacy-image-cache-stale-secs = キャッシュされた画像は、その鮮度を数秒経過しても、バックグラウンドで再検証されている間、同時に提供されます。ゼロはそれをオフにします。範囲は 0 ～ 86400。デフォルトは 3600。

admin-settings-help-privacy-image-resize-enabled = 特大のリモート画像をリーダーが要求する幅 (320、640、960、1280、または 1600 ピクセル) まで縮小します。アニメーション画像はスケールなしで提供されます。デフォルトではオフです。

admin-settings-help-privacy-image-resize-format = ビルド内で最適なエンコーダー (PNG) が自動的に選択されます。デフォルトの自動。

admin-settings-help-privacy-image-resize-max-pixels = これより多くのデコードされたピクセルを含む画像は、拡大縮小されずに提供されます。サイズはデコードの前にヘッダーから読み取られます。範囲は 100000 ～ 64000000。デフォルトは4000000です。

admin-settings-help-privacy-image-resize-max-width = サイズ変更によって生成される最も幅の広い画像。範囲 320 ～ 1600。デフォルトは1600です。

admin-settings-help-privacy-sessions-show-linked-origin = [設定] > [セキュリティ] > [セッション] で、リンク メールボックス アクセス セッションの元のアカウントに名前を付ける方法 (完全なアドレス、名前の最初の部分がマスクされたアドレス、または何も指定しない)。値はセッションの所有者にのみ表示され、他のユーザーには表示されません。デフォルト: 完全なアドレス。

admin-settings-help-properties-folder-enabled = フォルダー メニューとショートカットでプロパティを提供します。オフの場合、ダイアログは使用できません。

admin-settings-help-properties-folder-notes-enabled = 各フォルダーに説明やメモを追加できます。

admin-settings-help-properties-folder-notes-max-length = フォルダーメモに保持できる最大文字数。

admin-settings-help-properties-folder-overrides-allowed = ユーザーが単一のフォルダーに対してオーバーライドできる設定。リストされていないオーバーライドは非表示になり、保存された値は無視されます。

admin-settings-help-properties-folder-sections = フォルダーのプロパティ ダイアログのセクション: 一般、リモート、サイズ、オフライン、オーバーライド、メモ、アクション。リストにないセクションは非表示になります。

admin-settings-help-properties-folder-size-compute = サーバー上のフォルダーのサイズが判明したとき: ダイアログが開いたとき、またはバックグラウンドで、決して実行されません。大きなメールボックスをカウントすると、サーバーのスキャンにコストがかかります。

admin-settings-help-properties-headers-raw = メッセージのプロパティ ダイアログに生のヘッダーを一覧表示できるようにします。オフの場合、解析された概要のみが表示されます。

admin-settings-help-properties-message-enabled = メッセージ メニューとショートカットでプロパティを提供します。オフの場合、ダイアログは使用できません。

admin-settings-help-properties-message-permanent-delete = メッセージ ダイアログでメッセージを永久に削除できるかどうか: 許可、2 回確認、または拒否。訴訟ホールド中のメッセージは決して削除されません。この方法で削除すると元に戻すことはできません。

admin-settings-help-properties-message-sections = メッセージのプロパティ ダイアログのセクション: 概要、スコアリング、ヘッダー、添付ファイル、アクション。リストにないセクションは非表示になります。

admin-settings-help-properties-offline-clear-enabled = ユーザーが [プロパティ] ダイアログから単一フォルダーのオフライン キャッシュをクリアできるようにします。

admin-settings-help-properties-remote-location-visibility = ユーザーに表示されるサーバー側の場所の範囲: 完全 (ホストとパス)、マスクされたホスト (ホストの一部が非表示)、パスのみ (ホストのないメールボックス パス)、または非表示。

admin-settings-help-properties-scoring-show = メッセージのプロパティ ダイアログにスパム スコア、認証結果、セキュリティ シグナルを表示します。

admin-settings-help-properties-shortcut-enabled = キーボードから、フォーカスされたフォルダーまたは選択したメッセージの [プロパティ] ダイアログを開きます。

admin-settings-help-properties-shortcut-key = プロパティを開くキーの組み合わせ。デフォルトでは Alt+Enter です。

admin-settings-help-reader-actions-archive = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-block-sender = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-colors = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-conversation = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-create-filter = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-delete = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-delete-sender = デフォルトでは非表示になっています。送信者削除機能と機能プロファイルで許可されている場合は、ツールバーまたは [その他] メニューを選択して有効にします。ロックされていない限り、ユーザーは配置をオーバーライドできます。

admin-settings-help-reader-actions-edit-draft = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-export = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-finish-followup = フラグ付きメッセージやフォローアップ リマインダーを完了するには、ツールバー、[その他] メニュー、または非表示を選択します。

admin-settings-help-reader-actions-flag = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-focus-focused = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-focus-other = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-forward = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-headers = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-links = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-move = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-not-spam = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-open-draft = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-pin = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-print = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-properties = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-read = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-reply = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-reply-all = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-save-contacts = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-save-sender = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-search = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-security = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-spam = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-sweep = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-view-source = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-reader-actions-zoom = ツールバー、その他メニュー、または非表示を選択します。機能制限、権限、メッセージ コンテキストは引き続き適用されます。これがロックされていない限り、ユーザーは選択できます。

admin-settings-help-retention-client-errors-event-days = 個々のブラウザ エラーの発生 (スタック トレースとブラウザ クラス) が保持される日数。デフォルトは 30 です。グループはより長く保持されます。次の設定を参照してください。

admin-settings-help-retention-client-errors-group-days = ブラウザー エラー グループ (名前、メッセージ、カウント) が最後に確認されてから保持される日数。デフォルトは 180 です。オカレンス保持期間よりも短くすることはできません。

admin-settings-help-retention-metrics-day-buckets-days = 毎日のパフォーマンス統計が削除されるまでに保持される日数。 0 を指定すると、それらが無制限に保持されるため、サーバーが実行されている間、テーブルはゆっくりと成長します。

admin-settings-help-retention-metrics-hour-buckets-days = 日次行 (少なくとも 1 つ) にマージされる前に保持される時間ごとのパフォーマンス統計の日数。

admin-settings-help-retention-metrics-minute-buckets-hours = 時間ごとの行 (少なくとも 1 つ) にマージされる前に保持される 1 分間のパフォーマンス統計の時間。

admin-settings-help-retention-user-allow-strip-attachments = ユーザーが古いメールから大きな添付ファイルを削除するルールを作成できるかどうか。デフォルトではオフです。オフの場合、アクションはルール エディターで非表示になり、それを使用するルールは保存時に拒否され、以前に保存されたルールはジョブの実行時にスキップされます。

admin-settings-help-retention-user-calendar-enabled = 終了から設定された日数後にイベントが削除されるカレンダーをユーザーが選択できるかどうか。デフォルトではオフです。オフの場合、オプションは設定で非表示になり、選択内容は保存時に拒否され、以前に保存された選択内容は実行されません。

admin-settings-help-retention-user-calendar-max-days = 終了したイベントが削除されるまで、ユーザーはほとんどの日を待つことになります。より大きな値は保存時に拒否され、実行時に値が低くなります。

admin-settings-help-retention-user-calendar-min-days = イベント終了後、ユーザー ルールによってイベントが削除されるまでの最短日数。より小さい値は保存時に拒否され、実行時に値が上げられます。

admin-settings-help-retention-user-notes-enabled = 設定した日数変更されなかったメモをユーザーが削除できるかどうか。デフォルトではオフです。削除されたメモは、保持ルールの元に戻すウィンドウが終了するまで復元できます。

admin-settings-help-retention-user-notes-max-days = ユーザーは、ほとんどの場合、手付かずのメモが削除されるまで待つことになります。より大きな値は保存時に拒否され、実行時に値が低くなります。

admin-settings-help-retention-user-notes-min-days = ユーザー ルールによってメモが削除される前に、メモが何日も放置されていなければなりません。より小さい値は保存時に拒否され、実行時に値が上げられます。

admin-settings-help-retention-user-strip-keep-inline-images = 添付ファイルが削除されたときに、メッセージ本文内に表示される画像が保持されるかどうか。

admin-settings-help-retention-user-strip-min-age-days = ルールによって添付ファイルが削除される前に、メッセージが経過してから最短の日数が経過する必要があります。小さい値はルールの保存時に拒否され、実行時に発生します。

admin-settings-help-retention-user-strip-min-attachment-kb = 少なくともこのサイズの添付ファイルのみが削除されます。小さい添付ファイルはメッセージと一緒に残ります。

admin-settings-help-retention-user-strip-undo-days = 添付ファイルが削除された後、元のメッセージがゴミ箱に残る日数。 -1 は、保持ルールの元に戻すウィンドウに従います。 0 はオリジナルを一括削除します。

admin-settings-help-server-shutdown-grace-seconds = 停止しているサーバーが接続を閉じるまでに、実行中のリクエストを終了させる時間 (秒単位)。 MW_HTTP_SHUTDOWN_TIMEOUT_MS 変数はこれをオーバーライドします。次回起動時から適用されます。

admin-settings-help-session-last-activity-flush-seconds = セッションの最後に表示された時刻は、最大でもこの頻度 (秒単位) で書き込まれます。 Zero はリクエストごとにそれを書き込みます。アイドル制限または絶対制限は変更されません。次回起動時から適用されます。

admin-settings-help-session-persist-across-restarts = オンにすると、メールボックス、アプリ、およびコンソールのセッションはサーバーの再起動後も有効なままになります。オフの場合、サーバーの起動時にすべてのセッションが終了します。次回起動時から適用されます。

admin-settings-key-attachments-bar-click-to-toggle = 添付ファイルバーをクリックして切り替えます

admin-settings-key-attachments-bar-download-all = 「すべてダウンロード」ボタン

admin-settings-key-attachments-bar-show-download-buttons = 添付ファイルのダウンロードボタン

admin-settings-key-attachments-bar-zip-inline-images = すべてダウンロード ZIP のインライン画像

admin-settings-key-cap-reading-delete-sender = 送信者からのすべてのメールの削除を許可する

admin-settings-key-compose-feature-family = 作曲家の特徴

admin-settings-key-compose-headers-allow-auto-submitted = 自動送信を許可する

admin-settings-key-compose-headers-allow-custom = カスタムヘッダーを許可する

admin-settings-key-compose-headers-allow-return-receipt-to = 返送先の受信を許可する

admin-settings-key-compose-headers-allow-threading-edit = スレッドヘッダーの編集を許可する

admin-settings-key-compose-headers-forced = 強制ヘッダー

admin-settings-key-compose-insert-image-resize-default = 画像サイズのプリセット

admin-settings-key-compose-insert-max-inline-image-mb = 最大のインライン画像 (MB)

admin-settings-key-compose-layout-default-font = デフォルトのメッセージフォント

admin-settings-key-compose-reply-to-allow-override = 返信先の変更を許可する

admin-settings-key-compose-reply-to-show = 返信先行

admin-settings-key-compose-sent-folder-fixed-id = 送信コピー：固定フォルダー

admin-settings-key-compose-sent-folder-mode = 送信済みコピー: フォルダー ルール

admin-settings-key-compose-sent-folder-per-identity = 送信済みコピー: ID ごとのフォルダー

admin-settings-key-compose-sent-folder-rules = 送信済みコピー: ルール

admin-settings-key-compose-sent-folder-skip-copy = 送信済みコピー: 保存しないでください

admin-settings-key-compose-styles-custom = カスタムスタイル

admin-settings-key-compose-styles-default-set = スタイルセット

admin-settings-key-compose-styles-max-custom = ユーザーごとのカスタムスタイル

admin-settings-key-contacts-save-from-message-add-note = メッセージから保存された連絡先に関する注意事項

admin-settings-key-contacts-save-from-message-default-book = メッセージから保存された連絡先のアドレス帳

admin-settings-key-contacts-save-from-message-default-scope = メッセージから保存するときに最初にチェックマークが付けられる連絡先

admin-settings-key-contacts-save-from-message-enabled = メッセージから連絡先を保存する

admin-settings-key-contacts-save-from-message-skip-automated = 連絡先を保存するときに自動送信者をスキップする

admin-settings-key-contacts-save-sender-enabled = 送信者を連絡先として保存

admin-settings-key-contacts-save-sender-use-reply-to = リスト送信者の返信先を保存する

admin-settings-key-dsr-admin-step-up = 承認する前にもう一度確認してください

admin-settings-key-export-default-format = エクスポートダイアログのデフォルト形式

admin-settings-key-export-default-scope = エクスポートダイアログのデフォルトスコープ

admin-settings-key-export-dialog-enabled = エクスポートダイアログ

admin-settings-key-export-folder-enabled = フォルダーのエクスポート

admin-settings-key-export-folder-format = フォルダーのエクスポート形式

admin-settings-key-export-folder-include-subfolders = エクスポートにサブフォルダーを含める

admin-settings-key-export-folder-max-bytes = フォルダーのエクスポート サイズ制限

admin-settings-key-export-folder-max-messages = フォルダーエクスポートメッセージの制限

admin-settings-key-export-formats = ダイアログ形式のエクスポート

admin-settings-key-export-message-download-enabled = メッセージのダウンロード

admin-settings-key-folders-hide-empty = 空のフォルダーを非表示にする

admin-settings-key-folders-pinned-hide-from-tree = 固定されたフォルダーをツリーから非表示にする

admin-settings-key-mail-badge-auth = 認証バッジ

admin-settings-key-mail-badge-density = リーダーバッジのサイズ

admin-settings-key-mail-badge-folder = フォルダーバッジ

admin-settings-key-mail-badge-headers = ヘッダーバッジ

admin-settings-key-mail-badge-labels = 読者バッジのラベル

admin-settings-key-mail-badge-list = メーリングリストのバッジ

admin-settings-key-mail-badge-sanitized = サニタイズ済みの HTML バッジ

admin-settings-key-mail-badge-spam = スパムスコアバッジ

admin-settings-key-mail-brand-avatars-require-auth = ブランドロゴには認証が必要です

admin-settings-key-mail-compose-header-auto-collapse-seconds = Composer ヘッダーの自動折りたたみ遅延

admin-settings-key-mail-compose-priority-default = デフォルトのメッセージ優先度

admin-settings-key-mail-compose-priority-enabled = Composer の優先順位セレクター

admin-settings-key-mail-expiry-hide-expired = 期限切れのメッセージ

admin-settings-key-mail-list-keep = 常に保管してください

admin-settings-key-mail-list-page-size = メッセージリストのページサイズ

admin-settings-key-mail-list-unsubscribe-mode = メーリングリストの購読を解除する

admin-settings-key-mail-message-list-group = メッセージのグループ化

admin-settings-key-mail-priority-marks = 優先マーク

admin-settings-key-mail-priority-marks-style = 優先マークのスタイル

admin-settings-key-mail-priority-send-headers = 送信時に書き込まれる優先ヘッダー

admin-settings-key-mail-priority-trust-sender-headers = 送信者が設定した優先順位を信頼する

admin-settings-key-mail-reply-from-header-priority = 応答 ID を選択するヘッダー

admin-settings-key-mail-search-default-scope = デフォルトのメール検索範囲

admin-settings-key-metrics-dimensions-enabled = 統計をクライアントの種類とデバイスクラスごとに分割する

admin-settings-key-metrics-persist-enabled = 再起動後も統計を保持する

admin-settings-key-metrics-persist-flush-seconds = 統計情報の保存間隔（秒）

admin-settings-key-observability-client-capture-console = コンソールエラーを報告する

admin-settings-key-observability-client-capture-csp = CSP 違反を報告する

admin-settings-key-observability-client-enabled = ブラウザエラーレポート

admin-settings-key-observability-client-forward = 転送エラーレポート

admin-settings-key-observability-client-include-account-hash = エラーレポートにアカウントハッシュを追加

admin-settings-key-observability-client-max-per-session = ページ読み込みごとのエラーレポート

admin-settings-key-observability-client-sample-rate = エラーレポートのサンプルレート

admin-settings-key-observability-client-source-maps = ソースマップを使用してスタックトレースを解決する

admin-settings-key-privacy-error-reports = エラーレポートを送信する

admin-settings-key-privacy-image-cache-max-stale-secs = オリジンに障害が発生した場合に古いままにする (秒)

admin-settings-key-privacy-image-cache-stale-secs = 再検証中に古いものを提供する (秒)

admin-settings-key-privacy-image-resize-enabled = 大きな画像のサイズを変更する

admin-settings-key-privacy-image-resize-format = サイズ変更された画像フォーマット

admin-settings-key-privacy-image-resize-max-pixels = サイズ変更する最大の画像 (ピクセル)

admin-settings-key-privacy-image-resize-max-width = サイズ変更された画像の最大幅 (ピクセル)

admin-settings-key-privacy-sessions-show-linked-origin = セッション内のリンクされたメールボックス起点

admin-settings-key-properties-folder-enabled = フォルダーのプロパティダイアログ

admin-settings-key-properties-folder-notes-enabled = フォルダーのメモ

admin-settings-key-properties-folder-notes-max-length = フォルダーノートの長さ

admin-settings-key-properties-folder-overrides-allowed = フォルダーごとの上書きが許可される

admin-settings-key-properties-folder-sections = フォルダーダイアログのセクション

admin-settings-key-properties-folder-size-compute = フォルダーサイズの計算

admin-settings-key-properties-headers-raw = 生のヘッダー

admin-settings-key-properties-message-enabled = メッセージプロパティダイアログ

admin-settings-key-properties-message-permanent-delete = プロパティから完全に削除

admin-settings-key-properties-message-sections = メッセージダイアログのセクション

admin-settings-key-properties-offline-clear-enabled = フォルダーごとのオフライン キャッシュをクリアする

admin-settings-key-properties-remote-location-visibility = 遠隔地の詳細

admin-settings-key-properties-scoring-show = スコアリングと認証の詳細

admin-settings-key-properties-shortcut-enabled = プロパティのショートカット

admin-settings-key-properties-shortcut-key = プロパティのショートカットキー

admin-settings-key-reader-actions-archive = アーカイブアクションの配置

admin-settings-key-reader-actions-block-sender = 送信者アクションの配置をブロックする

admin-settings-key-reader-actions-colors = メッセージカラーアクションの配置

admin-settings-key-reader-actions-conversation = 会話アクションの配置を表示する

admin-settings-key-reader-actions-create-filter = フィルターアクションの配置を作成する

admin-settings-key-reader-actions-delete = アクションの配置を削除する

admin-settings-key-reader-actions-delete-sender = 送信者からすべてのメッセージを削除するアクション

admin-settings-key-reader-actions-edit-draft = ドラフトアクションの配置を編集する

admin-settings-key-reader-actions-export = アクションの配置をエクスポート

admin-settings-key-reader-actions-finish-followup = フォローアップを完了したアクションとしてマークする

admin-settings-key-reader-actions-flag = フラグ/フラグ解除アクションの配置

admin-settings-key-reader-actions-focus-focused = 集中的なアクションの配置に移動

admin-settings-key-reader-actions-focus-other = 他のアクションの配置に移動

admin-settings-key-reader-actions-forward = フォワードアクションの配置

admin-settings-key-reader-actions-headers = 生のヘッダーアクションの配置

admin-settings-key-reader-actions-links = リンクアクションの配置を確認する

admin-settings-key-reader-actions-move = アクションの配置を移動する

admin-settings-key-reader-actions-not-spam = スパムアクションの配置ではありません

admin-settings-key-reader-actions-open-draft = 返信の下書きアクションの配置を開く

admin-settings-key-reader-actions-pin = ピン留め/ピン留め解除アクションの配置

admin-settings-key-reader-actions-print = 印刷アクションの配置

admin-settings-key-reader-actions-properties = メッセージプロパティアクションの配置

admin-settings-key-reader-actions-read = 既読/未読アクションの配置

admin-settings-key-reader-actions-reply = 返信アクションの配置

admin-settings-key-reader-actions-reply-all = すべてのアクションの配置を返信

admin-settings-key-reader-actions-save-contacts = メッセージを保存する連絡先アクションの配置

admin-settings-key-reader-actions-save-sender = 送信者アクションの配置を保存する

admin-settings-key-reader-actions-search = 関連する検索アクションの配置

admin-settings-key-reader-actions-security = セキュリティモードアクションの配置

admin-settings-key-reader-actions-spam = スパムとしてマークするアクションの配置

admin-settings-key-reader-actions-sweep = スイープセンダーアクションの配置

admin-settings-key-reader-actions-view-source = メッセージソースアクションの配置を表示する

admin-settings-key-reader-actions-zoom = ズームアクションの配置

admin-settings-key-retention-client-errors-event-days = エラー発生の保持(日)

admin-settings-key-retention-client-errors-group-days = エラーグループの保持数(日)

admin-settings-key-retention-metrics-day-buckets-days = 保持される毎日の統計 (日)

admin-settings-key-retention-metrics-hour-buckets-days = 保持される時間ごとの統計 (日)

admin-settings-key-retention-metrics-minute-buckets-hours = 保持される分単位の統計 (時間)

admin-settings-key-retention-user-allow-strip-attachments = ユーザーは古いメールから添付ファイルを削除できます

admin-settings-key-retention-user-calendar-enabled = ユーザーは終了したカレンダーイベントを削除できます

admin-settings-key-retention-user-calendar-max-days = 終了したイベントが削除されるほとんどの日前

admin-settings-key-retention-user-calendar-min-days = 終了したイベントが削除されるまでの最短日数

admin-settings-key-retention-user-notes-enabled = ユーザーは未変更のメモを削除できます

admin-settings-key-retention-user-notes-max-days = 未変更のメモが削除されるまでのほとんどの日

admin-settings-key-retention-user-notes-min-days = 未変更のメモが削除されるまでの最短日数

admin-settings-key-retention-user-strip-keep-inline-images = インライン画像を保持する

admin-settings-key-retention-user-strip-min-age-days = 添付ファイルが削除されるまでの最短日数

admin-settings-key-retention-user-strip-min-attachment-kb = 削除された最小の添付ファイル (KB)

admin-settings-key-retention-user-strip-undo-days = 添付ファイルを削除した後の元に戻すウィンドウ (日)

admin-settings-key-server-shutdown-grace-seconds = シャットダウン猶予期間

admin-settings-key-session-last-activity-flush-seconds = 最後に確認された書き込み間隔

admin-settings-key-session-persist-across-restarts = 再起動後もセッションを維持する

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-mail-cache-keep-indefinitely = デフォルトではオフです。暗号化されたアカウント スコープのメール スナップショットをブラウザのリロード後も有効期限なしで保存します。既存のエントリおよびバイト制限は引き続き適用されます。ユーザーは保存されたコピーを消去できます。この設定を無効にすると、アクティブなブラウザーからそれらが削除されます。これにより、ブラウザーの永続ストレージ権限が付与されたり、添付ファイルをダウンロードしたりすることはありません。

admin-settings-help-mail-conversations-allowed = ユーザーが関連する返信を会話にグループ化できるようにします。デフォルトで利用可能ですが、最初はユーザー設定が優先されます。可用性を無効にすると、カテゴリのグループ化を変更せずに、ユーザーの選択が上書きされます。

admin-settings-help-mail-conversations-enabled = デフォルトではオフです。グループ化とは独立して、読み込まれた応答をスレッドごとにグループ化します。デフォルトを設定するか、この設定をロックします。会話の可用性も有効にする必要があります。

admin-settings-key-mail-cache-keep-indefinitely = ブラウザのメールキャッシュを無期限に保持する

admin-settings-key-mail-conversations-allowed = 会話スレッドを利用可能にする

admin-settings-key-mail-conversations-enabled = メッセージを会話にグループ化する

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-links-trusted-sender-domains = レビューされたメッセージ リンクが自動的に有効になる、正確な小文字の送信者ドメインが 100 個まで含まれます。サブドメインは別です。これは宛先 Web サイトを信頼せず、リモート画像も許可しません。デフォルトでは空です。ユーザーはロックされていない限り変更できます。

admin-settings-key-links-trusted-sender-domains = メッセージリンクの信頼できる送信者ドメイン

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-reader-attachment-action-style = ダウンロードとファイルの詳細をプレビュー タイトルの横にアイコン (デフォルト)、ラベル、またはアイコンとラベルとして表示します。グローバル、ドメインごと、またはユーザーごとに設定およびロックできます。

admin-settings-help-reader-attachment-thumbnails = デフォルトではオフです。表示される画像添付ファイルがサムネイル プレビューを読み込めるようにします。添付ファイルのセキュリティ ルールは引き続き適用されます。グローバル、ドメインごと、またはユーザーごとに設定およびロックできます。

admin-settings-key-reader-attachment-action-style = 添付ファイルのプレビューのアクション

admin-settings-key-reader-attachment-thumbnails = 画像添付ファイルのサムネイル

admin-settings-key-mail-compose-auto-add-mention-recipients = メンションした相手を自動的に宛先に追加する
admin-settings-help-mail-compose-auto-add-mention-recipients = @メンションの候補から相手を選択したとき、宛先、Cc、Bcc にまだ含まれていなければ宛先に追加します。既定では無効です。

admin-settings-linked-disable-title = リンクされたアカウントの保存済みアクセスを無効にしますか？
admin-settings-linked-disable-body = この変更を保存すると、リンクされたアカウントへのアクセス更新に使う認証情報が削除されます。アクセスを更新する際は再度ログインが必要です。この設定を再度有効にしても、削除された認証情報は復元されません。
admin-settings-linked-disable-confirm = 無効にして保存済みアクセスを削除

admin-settings-key-reader-auto-hide-header = メッセージのヘッダーを自動的に隠す
admin-settings-help-reader-auto-hide-header = 各メッセージを開く際、差出人、宛先、メタデータの詳細を折りたたみます。件名と操作は表示したままです。ヘッダーを表示で詳細を確認できます。既定では無効です。

admin-settings-key-time-region = 地域
admin-settings-help-time-region = auto または GB、US、PT などの2文字の地域コード。

admin-settings-key-time-date-format = 日付の形式
admin-settings-help-time-date-format = auto、short、medium、long、full、または yyyy-MM-dd や dd/MM/yyyy などのパターン。文字列は '引用符' で囲みます。

admin-settings-key-time-time-format = 時刻の形式
admin-settings-help-time-time-format = auto、short、medium、long、full、または HH:mm:ss / h:mm a。H = 24時間、h = 12時間、a = AM/PM。

admin-settings-key-time-hour-cycle = 時計
admin-settings-help-time-hour-cycle = カスタムパターンの H または h が優先されます。

admin-settings-key-time-first-day-of-week = 週の最初の曜日
admin-settings-help-time-first-day-of-week = 自動設定は地域の暦の慣習に従います。

admin-settings-key-compliance-mail-enabled = メールを有効にする
admin-settings-help-compliance-mail-enabled = TLS対応のSMTP中継を設定してください。メールはsystem_mailプロキシ経路を使用します。受理状況が不明な場合は再送前に確認してください。

admin-settings-key-compliance-mail-host = SMTP中継ホスト
admin-settings-help-compliance-mail-host = TLS対応のSMTP中継を設定してください。メールはsystem_mailプロキシ経路を使用します。受理状況が不明な場合は再送前に確認してください。

admin-settings-key-compliance-mail-port = SMTPポート
admin-settings-help-compliance-mail-port = TLS対応のSMTP中継を設定してください。メールはsystem_mailプロキシ経路を使用します。受理状況が不明な場合は再送前に確認してください。

admin-settings-key-compliance-mail-security = TLSモード
admin-settings-help-compliance-mail-security = TLS対応のSMTP中継を設定してください。メールはsystem_mailプロキシ経路を使用します。受理状況が不明な場合は再送前に確認してください。

admin-settings-key-compliance-mail-from = 送信者アドレス
admin-settings-help-compliance-mail-from = TLS対応のSMTP中継を設定してください。メールはsystem_mailプロキシ経路を使用します。受理状況が不明な場合は再送前に確認してください。

admin-settings-key-compliance-mail-username = SMTPユーザー名
admin-settings-help-compliance-mail-username = TLS対応のSMTP中継を設定してください。メールはsystem_mailプロキシ経路を使用します。受理状況が不明な場合は再送前に確認してください。

admin-settings-key-compliance-mail-allowPrivate = プライベート中継を許可
admin-settings-help-compliance-mail-allowPrivate = TLS対応のSMTP中継を設定してください。メールはsystem_mailプロキシ経路を使用します。受理状況が不明な場合は再送前に確認してください。

admin-settings-key-dsr-notify-emailAdmins = データ要求を運用者にメール通知
admin-settings-help-dsr-notify-emailAdmins = TLS対応のSMTP中継を設定してください。メールはsystem_mailプロキシ経路を使用します。受理状況が不明な場合は再送前に確認してください。

admin-settings-key-dsr-notify-operatorAddresses = 運用者の宛先
admin-settings-help-dsr-notify-operatorAddresses = TLS対応のSMTP中継を設定してください。メールはsystem_mailプロキシ経路を使用します。受理状況が不明な場合は再送前に確認してください。

admin-settings-key-dsr-notify-emailRequester = 要求者にメールで更新を通知
admin-settings-help-dsr-notify-emailRequester = TLS対応のSMTP中継を設定してください。メールはsystem_mailプロキシ経路を使用します。受理状況が不明な場合は再送前に確認してください。

admin-settings-key-breach-notify-email = 侵害を運用者にメール通知
admin-settings-help-breach-notify-email = TLS対応のSMTP中継を設定してください。メールはsystem_mailプロキシ経路を使用します。受理状況が不明な場合は再送前に確認してください。

admin-settings-key-breach-subjects-email = 影響を受ける人にメール通知
admin-settings-help-breach-subjects-email = TLS対応のSMTP中継を設定してください。メールはsystem_mailプロキシ経路を使用します。受理状況が不明な場合は再送前に確認してください。

admin-settings-key-compliance-mail-requireProxy = プロキシ経路を必須にする
admin-settings-help-compliance-mail-requireProxy = プロキシ経路を必須にする (system_mail).
admin-settings-key-retention-complianceMail-days = 完了したメール履歴の保存日数
admin-settings-help-retention-complianceMail-days = 完了した配信を指定日数保存します。待機中と受理状況不明の配信は解決するまで保持します。

admin-settings-key-incidents-auto-enabled = インシデント候補を自動検出
admin-settings-help-incidents-auto-enabled = 正常性チェックの失敗とセッションの警告からインシデント候補を自動作成します。重大性を評価し、復旧を検証してください。監視の空白だけでは停止の証拠になりません。
admin-settings-key-incidents-auto-failureSamples = 検出までの異常サンプル数
admin-settings-help-incidents-auto-failureSamples = 正常性チェックの失敗とセッションの警告からインシデント候補を自動作成します。重大性を評価し、復旧を検証してください。監視の空白だけでは停止の証拠になりません。
admin-settings-key-incidents-nis2-trustService = トラストサービスの報告期限を使用
admin-settings-help-incidents-nis2-trustService = 正常性チェックの失敗とセッションの警告からインシデント候補を自動作成します。重大性を評価し、復旧を検証してください。監視の空白だけでは停止の証拠になりません。
admin-settings-key-incidents-notify-email = インシデントの更新・リマインダーをメール送信
admin-settings-help-incidents-notify-email = 正常性チェックの失敗とセッションの警告からインシデント候補を自動作成します。重大性を評価し、復旧を検証してください。監視の空白だけでは停止の証拠になりません。
admin-settings-key-incidents-notify-operatorAddresses = 運用担当者のメール宛先
admin-settings-help-incidents-notify-operatorAddresses = 正常性チェックの失敗とセッションの警告からインシデント候補を自動作成します。重大性を評価し、復旧を検証してください。監視の空白だけでは停止の証拠になりません。
admin-settings-key-incidents-retention-days = 終了したインシデントの保持期間（日）
admin-settings-help-incidents-retention-days = 正常性チェックの失敗とセッションの警告からインシデント候補を自動作成します。重大性を評価し、復旧を検証してください。監視の空白だけでは停止の証拠になりません。

admin-settings-key-mail-snooze-presets = スヌーズの設定
admin-settings-help-mail-snooze-presets = プリセットの追加、名前変更、並べ替え、編集、非表示ができます。メニューでは常に日時を指定できます。

admin-settings-key-reader-actions-view-draft = 返信の下書きを表示
admin-settings-help-reader-actions-view-draft = 返信の下書きを表示

admin-settings-key-notifications-view-enabled = 通知ビューを有効にする
admin-settings-help-notifications-view-enabled = 通知センターの表示を制御します。セキュリティ通知、メール通知、ブラウザー通知はそれぞれの設定に従います。
admin-settings-key-notifications-view-show-in-sidebar = サイドバーに通知を表示する
admin-settings-help-notifications-view-show-in-sidebar = 通知センターの表示を制御します。セキュリティ通知、メール通知、ブラウザー通知はそれぞれの設定に従います。

admin-settings-key-mail-unified-account = 統合仮想アカウント
admin-settings-help-mail-unified-account = フォルダーと件数をまとめた仮想アカウントをアカウント選択欄に追加します。統合メッセージビューとは独立した設定です。

admin-settings-key-accounts-settings-inherit-groups = プライマリアカウントの設定を継承
admin-settings-help-accounts-settings-inherit-groups = 選択したグループは、このメールボックスに個別設定がない場合にプライマリアカウントの値を使用します。個別設定と管理者ポリシーが優先されます。テーマ、署名、認証情報、フォルダーIDは継承されません。

admin-settings-key-mail-badge-importance = 重要度
admin-settings-help-mail-badge-importance = 常に表示、重要な場合のみ、非表示から選択します。通常と異なる値、有効なリマインダー、確認、配信イベント、近い期限を強調表示します。メッセージのデータは保持されます。
admin-settings-key-mail-badge-follow-up = フォローアップ
admin-settings-help-mail-badge-follow-up = 常に表示、重要な場合のみ、非表示から選択します。通常と異なる値、有効なリマインダー、確認、配信イベント、近い期限を強調表示します。メッセージのデータは保持されます。
admin-settings-key-mail-badge-snooze = スヌーズ
admin-settings-help-mail-badge-snooze = 常に表示、重要な場合のみ、非表示から選択します。通常と異なる値、有効なリマインダー、確認、配信イベント、近い期限を強調表示します。メッセージのデータは保持されます。
admin-settings-key-mail-badge-receipts = 開封確認
admin-settings-help-mail-badge-receipts = 常に表示、重要な場合のみ、非表示から選択します。通常と異なる値、有効なリマインダー、確認、配信イベント、近い期限を強調表示します。メッセージのデータは保持されます。
admin-settings-key-mail-badge-delivery = 配信状態
admin-settings-help-mail-badge-delivery = 常に表示、重要な場合のみ、非表示から選択します。通常と異なる値、有効なリマインダー、確認、配信イベント、近い期限を強調表示します。メッセージのデータは保持されます。
admin-settings-key-mail-badge-dates = メッセージの日付
admin-settings-help-mail-badge-dates = 常に表示、重要な場合のみ、非表示から選択します。通常と異なる値、有効なリマインダー、確認、配信イベント、近い期限を強調表示します。メッセージのデータは保持されます。
admin-settings-key-mail-badge-sensitivity = 機密度
admin-settings-help-mail-badge-sensitivity = 常に表示、重要な場合のみ、非表示から選択します。通常と異なる値、有効なリマインダー、確認、配信イベント、近い期限を強調表示します。メッセージのデータは保持されます。
admin-settings-key-mail-badge-reply-by = 返信期限
admin-settings-help-mail-badge-reply-by = 常に表示、重要な場合のみ、非表示から選択します。通常と異なる値、有効なリマインダー、確認、配信イベント、近い期限を強調表示します。メッセージのデータは保持されます。
admin-settings-key-mail-badge-expires = 有効期限
admin-settings-help-mail-badge-expires = 常に表示、重要な場合のみ、非表示から選択します。通常と異なる値、有効なリマインダー、確認、配信イベント、近い期限を強調表示します。メッセージのデータは保持されます。
admin-settings-key-mail-badge-supersedes = 置き換えられたメッセージ
admin-settings-help-mail-badge-supersedes = 常に表示、重要な場合のみ、非表示から選択します。通常と異なる値、有効なリマインダー、確認、配信イベント、近い期限を強調表示します。メッセージのデータは保持されます。

admin-settings-key-mail-follow-up-presets = フォローアップのフラグ
admin-settings-help-mail-follow-up-presets = フォローアップ通知はスヌーズとは別に設定できます。フォローアップではメールを表示したままにし、スヌーズでは指定時刻まで非表示にします。管理者ポリシーが許可する場合、個別設定で継承値を変更できます。 既定で有効なのは「今日の後で」「明日」「来週」のみです。ほかのプリセットも有効化・編集できます。
