# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: crypto. Source: ../en/crypto.ftl

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = Criptografia de ponta a ponta
crypto-banner-e2ee-detail = Todos os destinatários têm uma chave — o corpo da mensagem é criptografado neste dispositivo.
crypto-banner-tls-label = Criptografia de transporte (TLS)
crypto-banner-tls-detail = Nenhuma chave de criptografia de destinatário foi encontrada; a entrega é protegida apenas em trânsito.
crypto-banner-mixed-label = Proteção mista
crypto-banner-mixed-detail = Alguns destinatários podem receber criptografia de ponta a ponta; outros recebem apenas TLS.
crypto-banner-checking-label = Verificando chaves dos destinatários
crypto-banner-checking = · verificando…
crypto-banner-tls-only = Apenas TLS para:

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = Segurança da mensagem
crypto-encrypt-label = Criptografar (de ponta a ponta)
crypto-encrypt-hint-default = Criptografar o corpo neste dispositivo antes de enviar.
crypto-encrypt-drafted = Rascunho criptografado neste dispositivo.
crypto-encrypt-no-key = Nenhuma chave de criptografia de destinatário disponível.
crypto-sign-label = Assinar (comprova que foi você que enviou)
crypto-protect-subject = Criptografar também o assunto
crypto-reason-add-recipient = Adicione um destinatário para verificar as chaves de criptografia.
crypto-reason-tls = Nenhuma chave de criptografia de destinatário disponível — enviando por TLS.

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = Avisos de prevenção contra perda de dados
crypto-dlp-block = Envio bloqueado
crypto-dlp-require = Criptografia obrigatória
crypto-dlp-warn = Atenção
crypto-dlp-notify = O administrador será notificado
crypto-dlp-matched = Correspondências: { $list }
