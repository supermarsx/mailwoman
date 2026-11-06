# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-requests. Source: ../en/admin-requests.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-history-not-found = バケットごとの未検出（404）
admin-history-nf-total = この期間の未検出（404）
admin-history-nf-share = リクエストの { $share }
admin-history-nf-not-tracked-window = この期間は 404 を記録していませんでした。
admin-history-nf-not-tracked-before = { $time } より前は 404 を記録していませんでした。それ以前の期間は、ゼロではなく利用不可と表示されます。
admin-history-nf-paths-title = 未検出が多いパス
admin-history-nf-paths-help = { $from } から { $to } まで、1時間ごとに最大 { $limit } 件のパスを、時間単位に切り上げ・切り下げて表示します。クエリ文字列は削除され、メールアドレス、数字、ID、トークン、長い値は {"{"}token{"}"} のようなプレースホルダーに置き換えられます。ブラウザーのアイコン要求など、アプリ自身が想定している未検出は集計されません。
admin-history-nf-paths-approximate = 保持できる数より多くの異なるパスが届いたため、一部の件数は正確な値ではなく範囲で表示されています。
admin-history-nf-paths-empty = この期間に未検出の応答はありません。
admin-history-nf-paths-error = 未検出のパスを読み込めませんでした。更新してもう一度お試しください。
admin-history-nf-paths-method = メソッド
admin-history-nf-paths-path = パス
admin-history-nf-paths-count = 件数
admin-history-nf-paths-range = { $min } 〜 { $max }
admin-history-nf-paths-client = アプリ自身のクライアントから
admin-history-nf-paths-client-help = ウェブアプリ、そのサービスワーカー、またはデスクトップ・モバイルアプリが要求するパスです。ビルドファイル、ワーカースクリプト、API、JMAP、DAV、コンソールのルートなどが含まれます。ここでの未検出は、通常、クライアントが古いか、ルートが間違っていることを意味します。
admin-history-nf-paths-external = その他のトラフィック
admin-history-nf-paths-external-help = PHP ファイル、ドットファイル、他のソフトウェアを探すスキャナーや、外部からのリンクなど、アプリが決して要求しないパスです。
admin-history-nf-paths-all = パス
admin-history-nf-paths-all-help = このサーバーは、クライアントのパスとその他のトラフィックを区別しません。
