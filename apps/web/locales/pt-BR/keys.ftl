# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: keys. Source: ../en/keys.ftl

# -- Panel header ------------------------------------------------------------
keys-panel-label = Gerenciamento de chaves
keys-title = Chaves e certificados
keys-subtitle = Chaves OpenPGP e S/MIME. As chaves privadas ficam neste dispositivo e nunca chegam ao servidor.
keys-generate = Gerar chave
keys-import = Importar chave

# -- Key list groups ---------------------------------------------------------
keys-your-keys = Suas chaves
keys-empty-own = Nenhuma chave ainda. Gere ou importe uma para começar.
keys-contact-keys = Chaves de contatos
keys-empty-contact = Nenhuma chave de contato. Pesquise uma abaixo.
keys-loading = Carregando chaves…
keys-select-prompt = Selecione uma chave para visualizá-la e verificá-la.
keys-untitled = Chave { $kind }

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = não verificada
keys-trust-tofu = tofu
keys-trust-verified = verificada
keys-trust-revoked = revogada

# -- Key detail --------------------------------------------------------------
keys-key-card = Chave { $name }
keys-fingerprint = Impressão digital
keys-safe-words = Palavras de verificação
keys-safe-words-help = Leia estas palavras em voz alta com o contato para confirmar a chave por outro canal.
keys-safe-words-list = Palavras de verificação da impressão digital
keys-scan-label = Escanear para verificar
keys-qr-label = Código QR da impressão digital
keys-autocrypt = Autocrypt
keys-autocrypt-on = Anunciada nos cabeçalhos Autocrypt
keys-autocrypt-off = Não anunciada via Autocrypt
keys-trust-label = Confiança
keys-trust-level = Nível de confiança
keys-associate-label = Associar a um contato
keys-associate-select = Contato a associar
keys-choose-contact = Escolha um contato…
keys-associate-button = Associar
keys-backup-label = Backup
keys-backup-help = Exporte uma Autocrypt Setup Message para levar esta chave a outro dispositivo.
keys-export-backup = Exportar backup
keys-asm-label = Autocrypt Setup Message
keys-no-private-backup = Nenhuma chave privada guardada neste dispositivo para backup

# -- Own S/MIME key: certificate, export, request, issued certificate ---------
keys-smime-section = Certificado S/MIME
# Shown for a key generated here (not for an imported or CA-issued certificate).
keys-smime-self-signed = Este certificado é autoassinado: foi assinado pela própria chave, e não por uma autoridade certificadora. Os programas de e-mail de outras pessoas não vão confiar nele, a menos que o aceitem manualmente ou até que uma autoridade certificadora emita um certificado a partir do arquivo de solicitação abaixo.
# $date is YYYY-MM-DD, taken from the certificate.
keys-smime-valid-until = Válido até { $date }.
keys-smime-not-in-app = O Mailwoman ainda não assina nem descriptografa mensagens com uma chave S/MIME. Exporte a chave para usá-la em outro programa de e-mail.
keys-smime-passphrase = Frase secreta desta chave S/MIME
keys-smime-export-help = O arquivo .p12 contém o certificado e a chave privada. A chave é criptografada com a frase secreta acima, e a mesma frase secreta abre o arquivo em outro programa. O arquivo .csr não contém chave privada.
keys-smime-export = Exportar como PKCS#12 (.p12)
keys-smime-request = Baixar solicitação de certificado (.csr)
keys-smime-issued-label = Certificado emitido para esta chave (.pem, .crt, .cer)
keys-smime-issued-file = Arquivo do certificado emitido
keys-smime-issued-help = Substitui o certificado guardado para esta chave. Um certificado feito para outra chave é recusado.
keys-smime-attach = Importar certificado emitido
keys-smime-failed = Não funcionou. Verifique a frase secreta.
keys-smime-attach-failed = O certificado não foi importado. Verifique a frase secreta e se o arquivo é um único certificado emitido para esta chave.

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = Pesquisar a chave de um contato
keys-lookup-heading = Pesquisar uma chave
keys-lookup-address = Endereço a pesquisar
keys-lookup-placeholder = name@example.org
keys-lookup-sources = Fontes de pesquisa
keys-source = Fonte { $source }
keys-consent-label = Consentimento para pesquisa externa
keys-consent-text = Pesquisar uma chave consulta diretórios externos (WKD/VKS). Tenho o consentimento desta pessoa para isso.
keys-lookup-button = Pesquisar
keys-looking-up = Pesquisando…
keys-lookup-found = { $count } chave(s) encontrada(s) — adicionada(s) às Chaves de contatos.
keys-lookup-none = Nenhuma chave encontrada.

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = Gerar uma chave
keys-type = Tipo
keys-key-type = Tipo de chave
keys-openpgp = OpenPGP
keys-smime = S/MIME
# Under the type list while S/MIME is chosen: what will be made, and its limits.
keys-generate-smime-note = Cria uma chave RSA de 3072 bits e um certificado para o endereço abaixo, válido por dois anos. O certificado é autoassinado: os programas de e-mail de outras pessoas não vão confiar nele, a menos que o aceitem manualmente ou até que uma autoridade certificadora emita um. O Mailwoman ainda não assina nem descriptografa mensagens com ele; depois você pode exportá-lo para outro programa de e-mail.
keys-name = Nome
keys-email = E-mail
keys-key-passphrase = Frase secreta da chave
keys-passphrase-help = A frase secreta protege a chave privada neste dispositivo. Ela nunca sai do navegador.
keys-cancel = Cancelar
keys-generate-submit = Gerar
keys-generating = Gerando…
keys-generating-smime = Gerando uma chave RSA neste navegador. Isso pode levar até um minuto.
keys-generate-failed = Nenhuma chave foi gerada.
keys-generate-failed-smime = Nenhuma chave foi gerada. Um certificado só pode conter um endereço em ASCII simples (nome@domínio) e um nome de no máximo 64 caracteres.

# -- Import dialog -----------------------------------------------------------
keys-import-title = Importar uma chave
keys-import-type = Tipo de importação
keys-tab-armored = Em texto ASCII (PGP)
keys-tab-pkcs12 = PKCS#12 (S/MIME)
keys-armored-key = Chave em texto ASCII
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = Frase secreta (se a chave for privada)
keys-import-passphrase = Frase secreta de importação
keys-pkcs12-file-label = Arquivo PKCS#12 (.p12 / .pfx)
keys-pkcs12-file = Arquivo PKCS#12
keys-import-password-label = Senha de importação
keys-pkcs12-password = Senha do PKCS#12
keys-import-preview = Pré-visualização da importação
keys-preview = Pré-visualizar
keys-preview-type = Tipo: { $kind }
keys-preview-fingerprint-aria = Impressão digital da pré-visualização
keys-preview-fingerprint = Impressão digital: { $fp }
keys-preview-has-private = Inclui uma chave privada (será guardada neste dispositivo)
keys-preview-public-only = Somente chave pública
keys-import-submit = Importar
