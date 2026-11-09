# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: avatars. Source: ../en/avatars.ftl


# -- Machine translation (2026-11): added to match en ---------------------
avatars-title = 連絡先とプロフィール画像
avatars-mode = 送信者と連絡先の画像
avatars-off = 画像を非表示
avatars-initials = イニシャルをローカルで生成
avatars-contacts = ローカルの連絡先画像を使用し、ない場合はイニシャルを使用
avatars-help = イニシャルはこのデバイス上で生成されます。連絡先画像には、自分のアドレス帳にすでに保存されている埋め込みの PNG または JPEG 写真のみを使用します（最大 512 × 512 ピクセル、128 KiB）。リモートの画像 URL や外部のアバターサービスにはアクセスしません。この設定はアカウントに保存され、管理者が固定することもできます。
avatars-default = 継承された設定に戻す
avatars-sender-icons = 組み込みの送信元ドメインアイコンを表示
avatars-sender-icons-help = 認識されたソーシャル、開発者向け、AI、ショッピング、政府のドメインには、このアプリに組み込まれたアイコンを使用します。自分の連絡先画像が優先されます。不明な送信者には、192 色からローカルで生成したイニシャルを使用します。アイコンはドメインの目印であり、メッセージが本物である証明ではありません。外部のロゴサービスには接続しません。
avatars-icons-default = 継承されたアイコン設定に戻す
avatars-brand-avatars = 認証済みの送信者の横にブランドロゴを表示
avatars-brand-avatars-help = リーダーでは、メッセージがその企業自身のドメインに対する DKIM と、アライメントされた DMARC のチェックに合格した場合にのみ、認識された企業のロゴが送信者の横に表示されます。それ以外の場合は通常のアバターを使用します。ロゴはこのアプリに組み込まれており、送信者には何も要求しません。
avatars-brand-avatars-default = 継承されたブランドロゴ設定に戻す
avatars-domain-icon = { $name } のドメインアイコン · このアイコンでは送信者の身元は確認されません
avatars-favorites-first = 宛先の候補でお気に入りの連絡先を先に表示
avatars-favorites-first-help = 宛先を入力すると、スターを付けた連絡先が他の候補より先に表示されます。変わるのは順序だけで、候補が非表示になることはありません。
avatars-favorites-first-default = 継承されたお気に入り設定に戻す

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
avatars-require-auth = ブランドロゴには認証が必要です

avatars-require-auth-always = 常に (チェックに合格する必要があります)

avatars-require-auth-default = 継承された認証設定を復元する

avatars-require-auth-help = 認識された企業のロゴが送信者の横のイニシャルに置き換わる場合を選択します。しない: 送信者のドメインが一致するたびに表示します。チェックが失敗した場合のみ (デフォルト): メッセージが失敗した場合 (DKIM または DMARC) を除き、表示します。常に: メッセージがその会社独自のドメインの DKIM と調整された DMARC チェックを通過した場合にのみ表示するため、まだチェックされていないメールにはイニシャルが表示されます。ドメインの一致には、mail.example.com などのサブドメインが含まれます。この設定はアカウントとともに保存され、管理者が保持できます。

avatars-require-auth-known-failure = チェックが失敗した場合のみ

avatars-require-auth-never = なし (ドメインのみで一致)

avatars-unverified-sender = 未確認の送信者 · このメッセージは認証チェックに合格しなかったため、ブランド アイコンは表示されません
