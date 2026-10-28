# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: keys. Source: ../en/keys.ftl

# -- Panel header ------------------------------------------------------------
keys-panel-label = Gestão de chaves
keys-title = Chaves e certificados
keys-subtitle = Chaves OpenPGP e S/MIME. As chaves privadas ficam neste dispositivo e nunca chegam ao servidor.
keys-generate = Gerar chave
keys-import = Importar chave

# -- Key list groups ---------------------------------------------------------
keys-your-keys = As suas chaves
keys-empty-own = Ainda não há chaves. Gere ou importe uma para começar.
keys-contact-keys = Chaves de contactos
keys-empty-contact = Não há chaves de contactos. Procure uma abaixo.
keys-loading = A carregar chaves…
keys-select-prompt = Selecione uma chave para a ver e verificar.
keys-untitled = Chave { $kind }

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = não verificada
keys-trust-tofu = TOFU
keys-trust-verified = verificada
keys-trust-revoked = revogada

# -- Key detail --------------------------------------------------------------
keys-key-card = Chave { $name }
keys-fingerprint = Impressão digital
keys-safe-words = Palavras de segurança
keys-safe-words-help = Leia-as em voz alta com o contacto para confirmar a chave por outro canal.
keys-safe-words-list = Palavras de segurança da impressão digital
keys-scan-label = Digitalizar para verificar
keys-qr-label = Código QR da impressão digital
keys-autocrypt = Autocrypt
keys-autocrypt-on = Anunciada nos cabeçalhos Autocrypt
keys-autocrypt-off = Não anunciada através de Autocrypt
keys-trust-label = Confiança
keys-trust-level = Nível de confiança
keys-associate-label = Associar a um contacto
keys-associate-select = Contacto a associar
keys-choose-contact = Escolha um contacto…
keys-associate-button = Associar
keys-backup-label = Cópia de segurança
keys-backup-help = Exporte uma Autocrypt Setup Message para mover esta chave para outro dispositivo.
keys-export-backup = Exportar cópia de segurança
keys-asm-label = Autocrypt Setup Message
keys-no-private-backup = Não há nenhuma chave privada neste dispositivo para a cópia de segurança

# -- Own S/MIME key: certificate, export, request, issued certificate ---------
keys-smime-section = Certificado S/MIME
# Shown for a key generated here (not for an imported or CA-issued certificate).
keys-smime-self-signed = Este certificado é autoassinado: foi assinado pela sua própria chave e não por uma autoridade de certificação. Os programas de correio de outras pessoas não confiarão nele, a menos que o aceitem manualmente ou até uma autoridade de certificação emitir um certificado a partir do ficheiro de pedido abaixo.
# $date is YYYY-MM-DD, taken from the certificate.
keys-smime-valid-until = Válido até { $date }.
keys-smime-not-in-app = O Mailwoman ainda não assina nem desencripta correio com uma chave S/MIME. Exporte a chave para a usar noutro programa de correio.
keys-smime-passphrase = Frase-passe desta chave S/MIME
keys-smime-export-help = O ficheiro .p12 contém o certificado e a chave privada. A chave é encriptada com a frase-passe acima, e a mesma frase-passe abre o ficheiro noutro programa. O ficheiro .csr não contém nenhuma chave privada.
keys-smime-export = Exportar como PKCS#12 (.p12)
keys-smime-request = Transferir pedido de certificado (.csr)
keys-smime-issued-label = Certificado emitido para esta chave (.pem, .crt, .cer)
keys-smime-issued-file = Ficheiro do certificado emitido
keys-smime-issued-help = Substitui o certificado guardado para esta chave. Um certificado criado para outra chave é recusado.
keys-smime-attach = Importar certificado emitido
keys-smime-failed = Não funcionou. Verifique a frase-passe.
keys-smime-attach-failed = O certificado não foi importado. Verifique a frase-passe e se o ficheiro é um único certificado emitido para esta chave.

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = Procurar a chave de um contacto
keys-lookup-heading = Procurar uma chave
keys-lookup-address = Endereço a procurar
keys-lookup-placeholder = name@example.org
keys-lookup-sources = Origens da pesquisa
keys-source = Origem { $source }
keys-consent-label = Consentimento para pesquisa externa
keys-consent-text = Procurar uma chave contacta diretórios externos (WKD/VKS). Tenho o consentimento desta pessoa para o fazer.
keys-lookup-button = Procurar
keys-looking-up = A procurar…
keys-lookup-found = Chaves encontradas: { $count } — adicionadas a «Chaves de contactos».
keys-lookup-none = Não foi encontrada nenhuma chave.

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = Gerar uma chave
keys-type = Tipo
keys-key-type = Tipo de chave
keys-openpgp = OpenPGP
keys-smime = S/MIME
# Under the type list while S/MIME is chosen: what will be made, and its limits.
keys-generate-smime-note = Cria uma chave RSA de 3072 bits e um certificado para o endereço abaixo, válido durante dois anos. O certificado é autoassinado: os programas de correio de outras pessoas não confiarão nele, a menos que o aceitem manualmente ou até uma autoridade de certificação emitir um. O Mailwoman ainda não assina nem desencripta correio com ele; depois, pode exportá-lo para outro programa de correio.
keys-name = Nome
keys-email = E-mail
keys-key-passphrase = Frase-passe da chave
keys-passphrase-help = A frase-passe protege a chave privada neste dispositivo. Nunca sai do navegador.
keys-cancel = Cancelar
keys-generate-submit = Gerar
keys-generating = A gerar…
keys-generating-smime = A gerar uma chave RSA neste navegador. Pode demorar até um minuto.
keys-generate-failed = Não foi gerada nenhuma chave.
keys-generate-failed-smime = Não foi gerada nenhuma chave. Um certificado só pode conter um endereço ASCII simples (nome@domínio) e um nome com no máximo 64 carateres.

# -- Import dialog -----------------------------------------------------------
keys-import-title = Importar uma chave
keys-import-type = Tipo de importação
keys-tab-armored = Armored (PGP)
keys-tab-pkcs12 = PKCS#12 (S/MIME)
keys-armored-key = Chave em formato armored
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = Frase-passe (se a chave for privada)
keys-import-passphrase = Frase-passe de importação
keys-pkcs12-file-label = Ficheiro PKCS#12 (.p12 / .pfx)
keys-pkcs12-file = Ficheiro PKCS#12
keys-import-password-label = Palavra-passe de importação
keys-pkcs12-password = Palavra-passe PKCS#12
keys-import-preview = Pré-visualização da importação
keys-preview = Pré-visualizar
keys-preview-type = Tipo: { $kind }
keys-preview-fingerprint-aria = Impressão digital pré-visualizada
keys-preview-fingerprint = Impressão digital: { $fp }
keys-preview-has-private = Inclui uma chave privada (será guardada neste dispositivo)
keys-preview-public-only = Apenas chave pública
keys-import-submit = Importar
