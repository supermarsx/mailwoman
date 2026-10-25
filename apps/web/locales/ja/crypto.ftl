# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: crypto. Source: ../en/crypto.ftl

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = エンドツーエンド暗号化
crypto-banner-e2ee-detail = すべての受信者が鍵を持っています — メッセージ本文はこのデバイスで暗号化されます。
crypto-banner-tls-label = 通信の暗号化（TLS）
crypto-banner-tls-detail = 受信者の暗号化鍵が見つかりませんでした。配信は通信経路上でのみ保護されます。
crypto-banner-mixed-label = 混在した保護
crypto-banner-mixed-detail = 一部の受信者はエンドツーエンド暗号化を受け取れます。それ以外の受信者は TLS のみです。
crypto-banner-checking-label = 受信者の鍵を確認中
crypto-banner-checking = · 確認中…
crypto-banner-tls-only = TLS のみの受信者：

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = メッセージのセキュリティ
crypto-encrypt-label = 暗号化（エンドツーエンド）
crypto-encrypt-hint-default = 送信前にこのデバイスで本文を暗号化します。
crypto-encrypt-drafted = 下書きはこのデバイスで暗号化されました。
crypto-encrypt-no-key = 受信者の暗号化鍵がありません。
crypto-sign-label = 署名（送信者があなたであることを証明）
crypto-protect-subject = 件名も暗号化する
crypto-reason-add-recipient = 暗号化鍵を確認するには受信者を追加してください。
crypto-reason-tls = 受信者の暗号化鍵がありません — TLS で送信します。

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = 情報漏えい防止の警告
crypto-dlp-block = 送信がブロックされました
crypto-dlp-require = 暗号化が必要です
crypto-dlp-warn = ご注意
crypto-dlp-notify = 管理者に通知されます
crypto-dlp-matched = 一致：{ $list }
