# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-telemetry. Source: ../en/admin-telemetry.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-telemetry-title = 運用テレメトリ
admin-telemetry-help = 運用トレースとメトリクスを、セルフホストの OpenTelemetry Collector など、OpenTelemetry 互換のコレクターにエクスポートします。エクスポートは既定でオフです。保存した変更は、サーバーの再起動後に反映されます。
admin-telemetry-back = オブザーバビリティに戻る
admin-telemetry-saved = テレメトリの設定を保存しました。変更を反映するにはサーバーを再起動してください。
admin-telemetry-restart = 保存済みの設定が、実行中のサーバーと異なります。再起動が必要です。
admin-telemetry-reload = 設定を再読み込み
admin-telemetry-save = テレメトリの設定を保存
admin-telemetry-collector = コレクターとシグナル
admin-telemetry-collector-help = このサーバー全体の送信先を選べるのは管理者のみです。プライベートなコレクターまたは HTTPS を使ってください。HTTP では、テレメトリが暗号化されずに送信されます。
admin-telemetry-enabled = OTLP エクスポートを有効にする
admin-telemetry-traces = トレースをエクスポート
admin-telemetry-metrics = メトリクスをエクスポート
admin-telemetry-endpoint = OTLP gRPC エンドポイント
admin-telemetry-endpoint-help = 絶対 URL の HTTP または HTTPS コレクターアドレスで、通常はポート 4317 です。認証情報、クエリ、フラグメントは含めないでください。これは gRPC エンドポイントであり、OTLP HTTP のポート 4318 ではありません。
admin-telemetry-serviceName = サービス名
admin-telemetry-serviceName-help = コレクター上で、この Mailwoman デプロイを識別するための固定のサービスラベルです。メールアドレスなどの個人情報は使わないでください。
admin-telemetry-environment = 環境
admin-telemetry-environment-help = production や staging などの任意のデプロイラベルです。エクスポートされるテレメトリと一緒に送信されます。
admin-telemetry-pinned = 無効になっている項目は、デプロイの環境変数で制御されています。値を上書きするには、それらの変数を変更してください。
admin-telemetry-limits = サンプリングとエクスポートの制限
admin-telemetry-limits-help = メモリとコレクターへのトラフィックを制限します。バッチサイズはキューサイズを超えられません。キューが満杯になると、メールのリクエストをブロックせずにスパンを破棄することがあります。
admin-telemetry-sampleRatio = トレースのサンプリング率
admin-telemetry-sampleRatio-help = 新しいルートトレースを保持する割合で、0 から 1 です。子スパンは親のサンプリング判断に従います。
admin-telemetry-exportTimeoutMs = エクスポートのタイムアウト（ミリ秒）
admin-telemetry-exportTimeoutMs-help = コレクターへのエクスポートリクエストの最大時間で、100 から 30000 ミリ秒です。
admin-telemetry-metricsIntervalSecs = メトリクスの間隔（秒）
admin-telemetry-metricsIntervalSecs-help = メトリクスのエクスポート間隔で、10 から 3600 秒です。
admin-telemetry-spanQueueSize = スパンキューのサイズ
admin-telemetry-spanQueueSize-help = エクスポート待ちのスパンの最大数で、128 から 32768 です。
admin-telemetry-spanBatchSize = スパンのバッチサイズ
admin-telemetry-spanBatchSize-help = 1回のエクスポートに含まれるスパンの最大数で、1 から 4096、かつキューサイズ以下です。
admin-telemetry-spanExportDelayMs = スパンのバッチ遅延（ミリ秒）
admin-telemetry-spanExportDelayMs-help = スパンのバッチ間の通常の待ち時間で、100 から 60000 ミリ秒です。
admin-telemetry-prometheus-help = Prometheus 互換のスクレイパー向けの、別の認証付きエンドポイントです。アクセスには常にデプロイの MW_METRICS_TOKEN ベアラートークンが必要で、その値がこのページに返されることはありません。
admin-telemetry-prometheusEnabled = Prometheus のスクレイピングを有効にする
admin-telemetry-token-missing = スクレイプ用トークンが設定されていません。運用担当者が MW_METRICS_TOKEN を設定するまで、メトリクスのエンドポイントにはアクセスできません。
