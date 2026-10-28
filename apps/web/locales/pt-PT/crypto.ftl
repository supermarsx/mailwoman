# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: crypto. Source: ../en/crypto.ftl

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = Encriptação ponto a ponto
crypto-banner-e2ee-detail = Todos os destinatários têm uma chave — o corpo da mensagem é encriptado neste dispositivo.
crypto-banner-tls-label = Encriptação no transporte (TLS)
crypto-banner-tls-detail = Não foram encontradas chaves de encriptação dos destinatários; a entrega só está protegida em trânsito.
crypto-banner-mixed-label = Proteção mista
crypto-banner-mixed-detail = Alguns destinatários podem receber encriptação ponto a ponto; os restantes recebem apenas TLS.
crypto-banner-checking-label = A verificar as chaves dos destinatários
crypto-banner-checking = · a verificar…
crypto-banner-tls-only = Apenas TLS para:

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = Segurança da mensagem
crypto-encrypt-label = Encriptar (ponto a ponto)
crypto-encrypt-hint-default = Encriptar o corpo neste dispositivo antes de enviar.
crypto-encrypt-drafted = Rascunho encriptado neste dispositivo.
crypto-encrypt-no-key = Não há nenhuma chave de encriptação disponível para os destinatários.
crypto-sign-label = Assinar (comprova que a mensagem é sua)
crypto-protect-subject = Encriptar também o assunto
crypto-reason-add-recipient = Adicione um destinatário para procurar chaves de encriptação.
crypto-reason-tls = Não há nenhuma chave de encriptação disponível para os destinatários — envio por TLS.

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = Avisos de prevenção de perda de dados
crypto-dlp-block = Envio bloqueado
crypto-dlp-require = Encriptação obrigatória
crypto-dlp-warn = Atenção
crypto-dlp-notify = O administrador será notificado
crypto-dlp-matched = Correspondências: { $list }
