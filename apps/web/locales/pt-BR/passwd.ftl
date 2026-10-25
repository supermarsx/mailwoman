# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: passwd. Source: ../en/passwd.ftl

# -- Region / heading --------------------------------------------------------
passwd-region-label = Alterar senha
passwd-heading = Alterar senha

# -- Forced change -----------------------------------------------------------
passwd-force-change = Seu administrador exige que você altere sua senha antes de continuar.

# -- Fields ------------------------------------------------------------------
passwd-current-label = Senha atual
passwd-new-label = Nova senha
passwd-confirm-label = Confirmar nova senha

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = As novas senhas coincidem.
passwd-match-no = As novas senhas ainda não coincidem.

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = pelo menos { $count } caracteres
passwd-rule-uppercase = uma letra maiúscula
passwd-rule-lowercase = uma letra minúscula
passwd-rule-digit = um dígito
passwd-rule-symbol = um símbolo

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = Esta conta é de acesso zero. Antes de a alteração ser aplicada, será exibida uma frase de recuperação — guarde-a para que você ainda consiga acessar seus dados se algo der errado.

# -- Actions -----------------------------------------------------------------
passwd-continue = Continuar
passwd-submit = Alterar senha

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = Guarde sua frase de recuperação
passwd-recovery-prose = Anote esta frase e guarde-a em um lugar seguro. Ela é exibida antes da alteração da senha para que você possa recuperar seus dados mesmo que a nova senha seja perdida. Ela não é armazenada no servidor.
passwd-recovery-ack-label = Guardei minha frase de recuperação
passwd-recovery-ack-text = Guardei minha frase de recuperação em um lugar seguro.

# -- Done phase --------------------------------------------------------------
passwd-done = Sua senha foi alterada.
passwd-done-reencrypt = Suas credenciais de servidor armazenadas foram criptografadas novamente com a nova senha.
passwd-done-rewrap = Suas chaves de acesso zero foram reencapsuladas com a nova senha.

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = digite sua senha atual
passwd-error-mismatch = a nova senha e a confirmação não coincidem
passwd-error-policy = a nova senha precisa ter { $rules }
passwd-error-ack-first = confirme primeiro que você guardou a frase de recuperação
passwd-error-prepare-recovery = não foi possível preparar a frase de recuperação
passwd-error-change = não foi possível alterar a senha
# Shown under the server's own message when a change is refused. Each line states
# what the status means; none of them claims the password was or was not changed.
passwd-error-session-ended = Sua sessão terminou. Entre novamente.
passwd-error-not-configured = Este servidor não está configurado para alterar senhas. Fale com o administrador.
passwd-error-contact-admin = O servidor não conseguiu concluir a alteração. Fale com o administrador.

# -- Forced change screen ----------------------------------------------------
# Shown instead of the mailbox while an administrator's "force password change"
# flag is set on the account. The server refuses mail requests until the change
# succeeds, so this copy must say exactly that — including the case where the
# server has no way to change the password.
passwd-forced-title = Altere sua senha para continuar
passwd-forced-signed-in-as = Conectado como { $username }.
passwd-forced-explain = Um administrador exigiu a alteração de senha desta conta. E-mail, agenda e contatos ficam indisponíveis até a alteração ser concluída.
passwd-forced-backend-note = Se a alteração for recusada mesmo com a senha atual correta, este servidor talvez não consiga alterar sua senha. Fale com o administrador: ele pode configurar a alteração de senhas ou remover a exigência.
passwd-forced-sign-out = Sair
passwd-forced-still-required = O servidor aceitou a alteração, mas ainda exige uma alteração de senha para esta conta. Fale com o administrador.
passwd-forced-reload-failed = A senha foi alterada, mas não foi possível carregar sua conta. Verifique sua conexão e tente novamente.
passwd-forced-retry = Tentar novamente
