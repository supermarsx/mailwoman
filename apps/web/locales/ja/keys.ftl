# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: keys. Source: ../en/keys.ftl

# -- Panel header ------------------------------------------------------------
keys-panel-label = 鍵の管理
keys-title = 鍵と証明書
keys-subtitle = OpenPGP と S/MIME の鍵。秘密鍵はこのデバイスに保存され、サーバーには送信されません。
keys-generate = 鍵を生成
keys-import = 鍵をインポート

# -- Key list groups ---------------------------------------------------------
keys-your-keys = 自分の鍵
keys-empty-own = 鍵はまだありません。生成またはインポートして開始してください。
keys-contact-keys = 連絡先の鍵
keys-empty-contact = 連絡先の鍵はありません。下で検索してください。
keys-loading = 鍵を読み込み中…
keys-select-prompt = 鍵を選択すると、表示と検証ができます。
keys-untitled = { $kind } 鍵

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = 未検証
keys-trust-tofu = TOFU
keys-trust-verified = 検証済み
keys-trust-revoked = 失効

# -- Key detail --------------------------------------------------------------
keys-key-card = 鍵 { $name }
keys-fingerprint = フィンガープリント
keys-safe-words = 確認用の単語
keys-safe-words-help = 連絡先の相手とこれらを読み上げ合い、別の経路で鍵を確認してください。
keys-safe-words-list = フィンガープリントの確認用単語
keys-scan-label = スキャンして検証
keys-qr-label = フィンガープリントの QR コード
keys-autocrypt = Autocrypt
keys-autocrypt-on = Autocrypt ヘッダーで公開中
keys-autocrypt-off = Autocrypt では公開していません
keys-trust-label = 信頼
keys-trust-level = 信頼レベル
keys-associate-label = 連絡先に関連付け
keys-associate-select = 関連付ける連絡先
keys-choose-contact = 連絡先を選択…
keys-associate-button = 関連付け
keys-backup-label = バックアップ
keys-backup-help = この鍵を別のデバイスに移すには、Autocrypt セットアップメッセージをエクスポートしてください。
keys-export-backup = バックアップをエクスポート
keys-asm-label = Autocrypt セットアップメッセージ
keys-no-private-backup = このデバイスにはバックアップできる秘密鍵がありません

# -- Own S/MIME key: certificate, export, request, issued certificate ---------
keys-smime-section = S/MIME 証明書
keys-smime-self-signed = この証明書は自己署名です。認証局ではなく、自身の鍵で署名されています。他の人のメールソフトは、手動で受け入れるか、下の要求ファイルから認証局が証明書を発行するまで、この証明書を信頼しません。
keys-smime-valid-until = 有効期限：{ $date }。
keys-smime-not-in-app = Mailwoman はまだ S/MIME 鍵でメールの署名や復号を行いません。別のメールソフトで使うには鍵をエクスポートしてください。
keys-smime-passphrase = この S/MIME 鍵のパスフレーズ
keys-smime-export-help = .p12 ファイルには証明書と秘密鍵が含まれます。鍵は上のパスフレーズで暗号化され、別のソフトでファイルを開くときも同じパスフレーズを使います。.csr ファイルには秘密鍵は含まれません。
keys-smime-export = PKCS#12（.p12）としてエクスポート
keys-smime-request = 証明書署名要求（.csr）をダウンロード
keys-smime-issued-label = この鍵に発行された証明書（.pem、.crt、.cer）
keys-smime-issued-file = 発行された証明書ファイル
keys-smime-issued-help = この鍵に保存されている証明書を置き換えます。別の鍵用に作られた証明書は拒否されます。
keys-smime-attach = 発行された証明書をインポート
keys-smime-failed = 処理できませんでした。パスフレーズを確認してください。
keys-smime-attach-failed = 証明書はインポートされませんでした。パスフレーズと、ファイルがこの鍵に発行された証明書 1 件であることを確認してください。

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = 連絡先の鍵を検索
keys-lookup-heading = 鍵を検索
keys-lookup-address = 検索するアドレス
keys-lookup-placeholder = name@example.org
keys-lookup-sources = 検索先
keys-source = 検索先 { $source }
keys-consent-label = 外部検索への同意
keys-consent-text = 鍵の検索では外部ディレクトリ（WKD/VKS）に問い合わせます。この相手から同意を得ています。
keys-lookup-button = 検索
keys-looking-up = 検索中…
keys-lookup-found = { $count } 件の鍵が見つかりました — 連絡先の鍵に追加しました。
keys-lookup-none = 鍵が見つかりませんでした。

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = 鍵の生成
keys-type = 種類
keys-key-type = 鍵の種類
keys-openpgp = OpenPGP
keys-smime = S/MIME
keys-generate-smime-note = RSA 3072 ビットの鍵と、下のアドレス用の証明書（有効期間 2 年）を作成します。証明書は自己署名です。他の人のメールソフトは、手動で受け入れるか認証局が証明書を発行するまで信頼しません。Mailwoman はまだこの鍵でメールの署名や復号を行いません。作成後、別のメールソフト用にエクスポートできます。
keys-name = 名前
keys-email = メールアドレス
keys-key-passphrase = 鍵のパスフレーズ
keys-passphrase-help = パスフレーズはこのデバイス上で秘密鍵を保護します。ブラウザーの外に送信されることはありません。
keys-cancel = キャンセル
keys-generate-submit = 生成
keys-generating = 生成中…
keys-generating-smime = このブラウザーで RSA 鍵を生成しています。最大で 1 分ほどかかることがあります。
keys-generate-failed = 鍵は生成されませんでした。
keys-generate-failed-smime = 鍵は生成されませんでした。証明書に使えるのは ASCII 文字のみのアドレス（name@domain）と、64 文字以内の名前だけです。

# -- Import dialog -----------------------------------------------------------
keys-import-title = 鍵のインポート
keys-import-type = インポートの種類
keys-tab-armored = ASCII 形式（PGP）
keys-tab-pkcs12 = PKCS#12（S/MIME）
keys-armored-key = ASCII 形式の鍵
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = パスフレーズ（秘密鍵の場合）
keys-import-passphrase = インポート用パスフレーズ
keys-pkcs12-file-label = PKCS#12 ファイル（.p12 / .pfx）
keys-pkcs12-file = PKCS#12 ファイル
keys-import-password-label = インポート用パスワード
keys-pkcs12-password = PKCS#12 パスワード
keys-import-preview = インポートのプレビュー
keys-preview = プレビュー
keys-preview-type = 種類：{ $kind }
keys-preview-fingerprint-aria = プレビューのフィンガープリント
keys-preview-fingerprint = フィンガープリント：{ $fp }
keys-preview-has-private = 秘密鍵を含みます（このデバイスに保存されます）
keys-preview-public-only = 公開鍵のみ
keys-import-submit = インポート
