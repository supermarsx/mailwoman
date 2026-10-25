# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: security. Source: ../en/security.ftl

# -- Security panel: sections + empty states --------------------------------
security-panel-region = メッセージのセキュリティ詳細
security-section-auth = 認証
security-section-received = 配送経路
security-section-signature = 署名
security-section-attachments = 添付ファイル
security-section-warnings = 警告
security-section-controls = 送信者の管理
security-received-empty = Received チェーンはありません。
security-attachments-empty = 添付ファイルはありません。
security-hop-unknown = 不明
security-unknown = 不明

# -- Authentication (DKIM/SPF/DMARC/ARC) results ----------------------------
security-auth-pass = 合格
security-auth-fail = 不合格
security-auth-none = なし
security-auth-neutral = 中立
security-auth-temperror = 一時的なエラー
security-auth-permerror = 恒久的なエラー

security-detail-domain = ドメイン
security-detail-selector = セレクター
security-detail-policy = ポリシー
security-detail-alignment = アライメント
security-detail-chain-length = チェーン長
security-aligned = 一致
security-not-aligned = 不一致

# -- Signature / certificate ------------------------------------------------
security-sig-verified = 署名を検証済み
security-sig-unverified-key = 署名あり — 署名者の鍵は未検証
security-sig-invalid = 署名が無効です
security-sig-none = 署名なし
security-chain-trusted = 信頼されたチェーン
security-chain-untrusted = 信頼されていないチェーン
security-chain-expired = チェーンの期限切れ
security-chain-unknown = チェーン不明
security-revocation-good = 失効していません
security-revocation-revoked = 鍵は失効しています
security-revocation-unknown = 失効状態は不明
security-key-changed = 前回から署名者の鍵が変わっています
security-fact-signer-key = 署名者の鍵
security-fact-algorithm = アルゴリズム
security-fact-key-created = 鍵の作成日
security-fact-key-expires = 鍵の有効期限
security-fact-chain = チェーン
security-fact-revocation = 失効
security-fact-key-change = 鍵の変更

# -- Attachment risk --------------------------------------------------------
security-attach-none = 既知のリスクなし
security-attach-macro = マクロを含む
security-attach-executable = 実行ファイル
security-attach-encrypted-archive = 暗号化されたアーカイブ
security-attach-double-extension = 二重拡張子
security-attach-mismatch = 種類の不一致（宣言：{ $declared }、検出：{ $detected }）

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = Reply-To アドレスが送信者と異なります
security-anomaly-envelopeFromDivergence = エンベロープの送信者が From アドレスと異なります
security-anomaly-messageIdDomainAnomaly = Message-ID のドメインが送信者と一致しません
security-anomaly-dateSkew = 送信日時がずれているようです
security-anomaly-punycodeSender = 送信者が punycode を使用しています（なりすましドメインの可能性）

# -- Sender controls --------------------------------------------------------
security-control-block = 送信者をブロック
security-control-silence = 送信者をミュート
security-control-ignore-conversation = スレッドを無視
security-control-report-phishing = フィッシングを報告
security-control-report-junk = 迷惑メールを報告
security-control-done-block = 送信者をブロックしました
security-control-done-silence = 送信者をミュートしました
security-control-done-ignore-conversation = スレッドを無視しました
security-control-done-report-phishing = フィッシングとして報告しました
security-control-done-report-junk = 迷惑メールとして報告しました

security-za-title = ゼロアクセスストレージ
security-za-on = オン
security-za-off = オフ
security-za-protects = ゼロアクセスは、保存されたメール、メモ、PIM データを暗号化し、ホスティングサーバーには読めない暗号文だけが残るようにします。これは、好奇心を持った運用者やストレージホストへの侵入から、保存データの内容を守ります。データベース、ディスク上のファイル、盗まれたバックアップを読んだ人が目にするのは XChaCha20-Poly1305 の暗号文であり、メッセージ本文、件名、添付ファイル、メモの本文、検索インデックスを復元することはできません。

security-za-server-sees-title = サーバーが引き続き見られるもの
security-za-sees-1 = 暗号文のデータ — 暗号化された行そのもの。
security-za-sees-2 = 不透明な行 ID — 行の保存と取得に使う識別子。
security-za-sees-3 = サイズ — 各暗号文の長さ（メッセージや添付ファイルのおおよそのサイズ）。
security-za-sees-4 = タイムスタンプ — 行が書き込まれ、更新された日時。
security-za-sees-5 = IMAP/SMTP の中継に必要なエンベロープの配送メタデータ — サーバーは引き続きあなたに代わって上流のメールプロバイダーに接続するため、接続のメタデータと、メールの送受信に必要な配送エンベロープはサーバーを通過します。

security-za-active-server-caveat = ゼロアクセスは、好奇心を持った、または侵入されたホストから「保存中の」データを守ります。ライブの IMAP/SMTP 通信を中継する、完全に悪意のある能動的なサーバーはより強力な攻撃者であり、ゼロアクセスはそれを防ぎません。そのようなサーバーはメールプロバイダーとのライブ接続上にあり、保存されたコピーがどのように暗号化されていても、通過するメールを観察したり改ざんしたりできます。ゼロアクセスは、「ストレージホストを運用する人（または侵入した人）に保存されたメールを読まれたくない」という懸念がある場合に選択してください。ライブのメール経路を不正に操作する運用者への防御にはなりません。

security-za-no-search-claim = ここにはサーバー側の検索可能暗号化はなく、そのような主張もしていません。検索は、ローカルで復号した内容に対してクライアント上だけで実行されます。サーバーが平文の検索可能な形式を保持することはありません。

security-za-recovery-tradeoff = 鍵はあなたのデバイス上でのみ導出・保持され、サーバーが平文の鍵を受け取ることはありません。パスフレーズ（またはパスキー）、ペアリングしたすべてのデバイス、リカバリーフレーズの「すべて」を失うと、暗号化されたデータは復元できません。リカバリーフレーズはオフラインで保存し、アカウントのマスターシークレットと同等に扱ってください。

security-za-setup-key = 鍵を設定
security-za-key-source = 鍵の種類
security-za-passphrase = パスフレーズ
security-za-passkey = パスキー（パスワードレス）
security-za-passkey-unavailable = このブラウザーではパスキーを利用できません
security-za-passphrase-label = ゼロアクセスのパスフレーズ
security-za-enable = ゼロアクセスを有効にする
security-za-recovery-title = リカバリーフレーズ
security-za-recovery-heading = リカバリーフレーズ — 今すぐオフラインで保存してください
security-za-recovery-note = これが唯一のコピーです。これを持っている人は誰でもあなたのデータを読めます。これ（とペアリングしたデバイス）がなければ、データは復元できません。
security-za-disable-section = ゼロアクセスを無効にする
security-za-disable-heading = ゼロアクセスをオフにする
security-za-disable-note = 新しいデータは再び暗号化されずに保存されます。既存の暗号化データは、鍵を導出できる間だけ読むことができます。
security-za-disable-btn = ゼロアクセスを無効にする
security-za-err-passphrase-len = 8 文字以上のパスフレーズを使用してください
security-za-err-enable = ゼロアクセスを有効にできませんでした
security-za-err-disable = ゼロアクセスを無効にできませんでした

security-pair-title = デバイスのペアリング
security-pair-heading = デバイスをペアリング
security-pair-intro = 鍵をサーバー経由で送信せずに、別のデバイスへ移します。サーバーが中継するのは、中身の読めない封印されたエンベロープだけです。ペアリングを信頼する前に、両方の画面に表示される 6 つの単語を比べてください。
security-pair-role = ペアリングの役割
security-pair-new-role = これが新しいデバイスです
security-pair-existing-role = ここから別のデバイスをペアリング
security-pair-show-qr = ペアリング QR を表示
security-pair-scan = 既存のデバイスでこれをスキャンしてください
security-pair-qr-label = デバイスのペアリング QR コード
security-pair-qr-default = ペアリング QR コード
security-pair-copy-code = または、このペアリングコードをコピー
security-pair-paste-envelope = もう一方のデバイスの封印されたエンベロープを貼り付けてください
security-pair-envelope-placeholder = エンベロープ…
security-pair-envelope-label = 封印されたエンベロープ
security-pair-complete = ペアリングを完了
security-pair-code-from-new = 新しいデバイスのペアリングコード
security-pair-code-placeholder = ペアリングコード…
security-pair-code-label = ペアリングコード
security-pair-seal = そのデバイス用に鍵を封印
security-pair-envelope-out = 封印されたエンベロープ — 新しいデバイスに貼り付けてください
security-pair-compare = 両方のデバイスでこれらの単語を比べてください
security-pair-sas-label = 短い認証文字列
security-pair-match = 単語が一致しています
security-pair-abort = 一致しません — 中止
security-pair-confirmed = ペアリングを確認しました。このチャネルは認証済みです。
security-pair-err-start = ペアリングを開始できませんでした
security-pair-err-complete = ペアリングを完了できませんでした
security-pair-err-unlock = 別のデバイスをペアリングするには、先にゼロアクセスのロックを解除してください
security-pair-err-seal = ルート鍵を封印できませんでした
