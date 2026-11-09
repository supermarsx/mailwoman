# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-privacy-speed. Source: ../en/admin-privacy-speed.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-speed-title = 速度
privacy-speed-unavailable = このサーバーからは速度の数値を取得できません。
privacy-speed-empty = この期間にはまだ画像が配信されていません。
privacy-speed-off = 速度の計測はオフです（設定：画像プロキシの速度計測）。新しい記録は残りません。
privacy-speed-verdict-healthy = 良好
privacy-speed-verdict-slow = 低速
privacy-speed-verdict-degraded = 性能低下
privacy-speed-verdict-noData = データ不足
privacy-speed-reason-ok = 取得した画像とキャッシュヒットは下記の上限内で、失敗したリクエストも少数です。
privacy-speed-reason-failureRate = リクエストの { $failurePct } % が、ビジー、タイムアウト、または上流の失敗で終わりました。
privacy-speed-reason-missP95 = 取得が必要だった画像が遅くなっています。95 パーセンタイルが下記の上限を超えています。
privacy-speed-reason-hitP95 = キャッシュ済みの画像の応答が遅くなっています。95 パーセンタイルが下記の上限を超えています。
privacy-speed-reason-fewRequests = この期間のリクエストが { $min } 件未満で、判断するには少なすぎます。
privacy-speed-thresholds = 低速と判定する条件：取得した画像の 95 パーセンタイルが { $slowMiss }、キャッシュ済み画像の 95 パーセンタイルが { $slowHit }、またはビジー・タイムアウト・失敗が { $slowFail } %。性能低下と判定する条件：取得した画像の 95 パーセンタイルが { $degradedMiss }、または失敗が { $degradedFail } %。上限は固定で、設定では変更できません。
privacy-speed-median = 中央値
privacy-speed-p95 = 95 パーセンタイル
privacy-speed-p99 = 99 パーセンタイル
privacy-speed-max = 最も遅い値
privacy-speed-requests = 計測したリクエスト
privacy-speed-busy = ビジー応答
privacy-speed-timeouts = タイムアウト
privacy-speed-upstream-errors = 上流のエラー
privacy-speed-hit = キャッシュヒット
privacy-speed-miss = 取得した画像（キャッシュミス）
privacy-speed-split-count = { $count } 件のリクエスト
privacy-speed-split-times = 中央値 { $p50 }、95 パーセンタイル { $p95 }、最も遅い値 { $max }。
privacy-speed-chart = 画像の配信にかかった時間
privacy-speed-chart-miss = 取得した画像の配信にかかった時間
privacy-speed-chart-unit = リクエスト受信から応答準備完了まで
privacy-speed-phases = 時間の内訳
privacy-speed-phases-help = キャッシュヒットではなかったリクエストについて、各ステップにかかった時間と、全体に占める割合です。
privacy-speed-phase = ステップ
privacy-speed-share = 時間の割合
privacy-speed-phase-queue = 処理の空き待ち
privacy-speed-phase-cache = キャッシュの検索
privacy-speed-phase-upstream = 上流からの取得
privacy-speed-phase-decode = デコードと再エンコード
privacy-speed-phase-other = その他すべて
privacy-speed-note = このサーバープロセスのみで計測し、メモリ上に保持されるため、再起動後は空になります。パーセンタイルの精度は { $error } % 以内です。URL、ホスト、アカウントは保存されません。

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-work-coalesced = 結合フェッチと優先リクエスト

privacy-work-coalesced-series = フェッチが飛行中にフェッチに結合されました

privacy-work-handle = ハンドルキャッシュヒット率

privacy-work-handle-series = キャッシュヒットの処理

privacy-work-per-interval = サンプリング間隔ごと

privacy-work-percent = ハンドル検索のシェア

privacy-work-priority-series = 優先リクエスト

privacy-work-resized = 縮小された画像

privacy-work-resized-series = サイズ変更

privacy-work-revalidated-series = 再検証されました

privacy-work-saved = ダウンスケーリングによって節約されたバイト数

privacy-work-saved-series = 保存されたバイト数

privacy-work-stale = 古いイメージが提供され、再検証される

privacy-work-stale-series = 古いものを提供する

privacy-work-title = 作業の回避と再形成
