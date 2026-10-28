# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: passwd. Source: ../en/passwd.ftl

# -- Region / heading --------------------------------------------------------
passwd-region-label = Alterar palavra-passe
passwd-heading = Alterar palavra-passe

# -- Forced change -----------------------------------------------------------
passwd-force-change = O seu administrador exige que altere a palavra-passe antes de continuar.

# -- Fields ------------------------------------------------------------------
passwd-current-label = Palavra-passe atual
passwd-new-label = Nova palavra-passe
passwd-confirm-label = Confirmar nova palavra-passe

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = As novas palavras-passe coincidem.
passwd-match-no = As novas palavras-passe ainda não coincidem.

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = pelo menos { $count } carateres
passwd-rule-uppercase = uma letra maiúscula
passwd-rule-lowercase = uma letra minúscula
passwd-rule-digit = um algarismo
passwd-rule-symbol = um símbolo

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = Esta conta é de acesso zero. Antes de a alteração ser aplicada, ser-lhe-á mostrada uma frase de recuperação — guarde-a para que possa continuar a aceder aos seus dados caso algo corra mal.

# -- Actions -----------------------------------------------------------------
passwd-continue = Continuar
passwd-submit = Alterar palavra-passe

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = Guarde a sua frase de recuperação
passwd-recovery-prose = Escreva esta frase e guarde-a num local seguro. É mostrada antes da alteração da palavra-passe para que possa recuperar os seus dados mesmo que a nova palavra-passe se perca. Não é armazenada no servidor.
passwd-recovery-ack-label = Guardei a minha frase de recuperação
passwd-recovery-ack-text = Guardei a minha frase de recuperação num local seguro.

# -- Done phase --------------------------------------------------------------
passwd-done = A sua palavra-passe foi alterada.
passwd-done-reencrypt = As credenciais de servidor armazenadas foram novamente encriptadas com a nova palavra-passe.
passwd-done-rewrap = As suas chaves de acesso zero foram novamente envolvidas com a nova palavra-passe.

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = introduza a sua palavra-passe atual
passwd-error-mismatch = a nova palavra-passe e a respetiva confirmação não coincidem
passwd-error-policy = a nova palavra-passe tem de ter { $rules }
passwd-error-ack-first = confirme primeiro que guardou a frase de recuperação
passwd-error-prepare-recovery = não foi possível preparar a frase de recuperação
passwd-error-change = não foi possível alterar a palavra-passe
# Shown under the server's own message when a change is refused. Each line states
# what the status means; none of them claims the password was or was not changed.
passwd-error-session-ended = A sua sessão terminou. Inicie sessão novamente.
passwd-error-not-configured = Este servidor não está configurado para alterar palavras-passe. Contacte o seu administrador.
passwd-error-contact-admin = O servidor não conseguiu concluir a alteração. Contacte o seu administrador.

# -- Forced change screen ----------------------------------------------------
# Shown instead of the mailbox while an administrator's "force password change"
# flag is set on the account. The server refuses mail requests until the change
# succeeds, so this copy must say exactly that — including the case where the
# server has no way to change the password.
passwd-forced-title = Altere a sua palavra-passe para continuar
passwd-forced-signed-in-as = Sessão iniciada como { $username }.
passwd-forced-explain = Um administrador exigiu a alteração da palavra-passe desta conta. O correio, o calendário e os contactos só ficam disponíveis depois de a alteração ser concluída com êxito.
passwd-forced-backend-note = Se a alteração for recusada apesar de a palavra-passe atual estar correta, este servidor pode não conseguir alterar a sua palavra-passe. Contacte o seu administrador: pode configurar a alteração de palavras-passe ou remover este requisito.
passwd-forced-sign-out = Terminar sessão
passwd-forced-still-required = O servidor aceitou a alteração, mas continua a exigir uma alteração da palavra-passe desta conta. Contacte o seu administrador.
passwd-forced-reload-failed = A palavra-passe foi alterada, mas não foi possível carregar a sua conta. Verifique a ligação e tente novamente.
passwd-forced-retry = Tentar novamente
