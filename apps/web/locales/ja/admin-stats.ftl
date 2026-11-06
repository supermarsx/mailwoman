# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-stats. Source: ../en/admin-stats.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-stats-key-figures = 主な数値
admin-stats-charts = グラフ
admin-stats-tables = 表
admin-stats-group-traffic = この期間のトラフィック
admin-stats-group-storage = ストレージと配信（現在）
admin-stats-refresh = 統計を更新
admin-stats-loading = 統計を読み込み中…
admin-stats-requests-total = リクエスト
admin-stats-errors-total = サーバーエラー（5xx）
admin-stats-p95-peak = 最も遅い 95 パーセンタイル
admin-stats-requests = リクエスト
admin-stats-errors = サーバーエラー（5xx）
admin-stats-not-found = 未検出（404）
admin-stats-latency = 応答時間
admin-stats-series-p50 = 中央値
admin-stats-series-p95 = 95 パーセンタイル
admin-stats-unit-count = { $bucket }あたりの件数
admin-stats-unit-duration = { $bucket }あたりの時間（表示の単位）
admin-stats-unit-accounts = ドメインごとのブロックされたアカウント
admin-stats-chart-about = { $from } から { $to } までの折れ線グラフ。
admin-stats-bars-about = { $count ->
   *[other] { $count } 個のドメインの棒グラフ。
}
admin-stats-view-table = 表で表示
admin-stats-legend = 凡例
admin-stats-empty-chart = この期間にはまだ何も計測されていません。より長い期間を選ぶか、サーバーがいくつかのリクエストを処理した後でもう一度確認してください。
admin-stats-empty-blocked = ブロックされているアカウントはありません。
admin-stats-delta-vs = 前の期間との比較
admin-stats-delta-same = 前の期間と同じ
admin-stats-delta-sr-up = 前の期間より { $change } 増加。
admin-stats-delta-sr-down = 前の期間より { $change } 減少。
admin-stats-delta-sr-same = 前の期間と同じ。
admin-ct-group = クライアントの読み込み時間
admin-ct-metric-timeToUsable = 操作可能になるまでの時間
admin-ct-metric-ttfb = 最初のバイト
admin-ct-metric-dcl = DOM 準備完了
admin-ct-metric-load = ページの読み込み
admin-ct-metric-fcp = First Contentful Paint
admin-ct-metric-lcp = Largest Contentful Paint
admin-ct-metric-folderOpen = フォルダーを開く
admin-ct-metric-folderLoadCold = フォルダーの読み込み（ネットワークから）
admin-ct-metric-folderLoadCached = フォルダーの読み込み（キャッシュから）
admin-ct-metric-messageLoadCold = メッセージの読み込み（ネットワークから）
admin-ct-metric-messageLoadCached = メッセージの読み込み（キャッシュまたは先読みから）
admin-ct-device-desktop = デスクトップ
admin-ct-device-mobile = モバイル
admin-ct-device-unknown = 不明
admin-ct-tile = { $metric }、{ $stat }
admin-ct-samples =
    { $count ->
       *[other] { $count } 件のサンプル
}
admin-ct-no-samples = この期間のサンプルはありません。
admin-ct-few-samples = サンプルは { $count } 件のみです。最低 { $min } 件が必要です。
admin-ct-chart-usable = 操作可能になるまでの時間
admin-ct-group-folder = フォルダーの読み込み時間
admin-ct-group-message = メッセージの読み込み時間
admin-ct-chart-folder-cold = ネットワークからのフォルダー読み込み
admin-ct-chart-folder-cached = キャッシュからのフォルダー読み込み
admin-ct-chart-message-cold = ネットワークからのメッセージ読み込み
admin-ct-chart-message-cached = キャッシュまたは先読みからのメッセージ読み込み
admin-ct-folder-off = フォルダーの読み込み時間は既定でオフです。オンにした人のみ計測されます。
admin-ct-message-off = メッセージの読み込み時間は既定でオフです。オンにした人のみ計測されます。
admin-ct-chart-note = パーセンタイルは、サンプルが { $min } 件以上ある区間のみ表示されます。
admin-ct-empty-chart = 中央値を表示するのに必要な { $min } 件のサンプルがある区間はありません。より長い期間を選んでください。
admin-ct-col-samples = サンプル
admin-ct-col-metric = 指標
admin-ct-col-device = デバイス
admin-ct-table-title = デバイス別のクライアント読み込み時間
admin-ct-table-caption = 指標とデバイスの種類ごとの、サンプル数、中央値、95 パーセンタイルです。
admin-ct-empty-table = この期間にブラウザーの読み込み時間は報告されていません。
admin-ct-failed = クライアントの読み込み時間を読み込めませんでした。
admin-ct-disabled = クライアントの読み込み時間の収集はオフになっています。以下の数値は、オフにする前のものです。
admin-ct-note = サインイン済みのブラウザーのみが対象で、セッションの { $rate }% をサンプリングし、匿名かつ集計済みです。操作可能になるまでの時間は、ナビゲーション開始からメッセージリストに最初の行が表示されるまでです。数値は推定値で、真の値との差は約 { $error }% 以内です。
