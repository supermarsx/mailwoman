# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-ux. Source: ../en/admin-ux.ftl

admin-ux-close = Fechar

# -- Page states (UX11) ---------------------------------------------------------
# $reason — a sentence saying why the data is not there.
admin-ux-state-unavailable = Indisponível — { $reason }
admin-ux-unavailable-default = este servidor ainda não oferece isso.
# $code — the server's error code or "HTTP <status>".
admin-ux-state-code = Código de erro: { $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = Abrir
admin-ux-list-back = Voltar à lista

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = Nenhuma alteração não salva
admin-ux-form-changes =
    { $count ->
        [one] 1 alteração não salva
       *[other] { $count } alterações não salvas
    }
admin-ux-form-discard = Descartar alterações
admin-ux-form-saving = Salvando…
admin-ux-form-saved =
    { $count ->
        [one] 1 alteração salva.
       *[other] { $count } alterações salvas.
    }
admin-ux-form-fix =
    { $count ->
        [one] Nada foi salvo. Corrija o campo marcado abaixo.
       *[other] Nada foi salvo. Corrija os { $count } campos marcados abaixo.
    }
admin-ux-learn-more = Saiba mais
# Added for screen readers after "Learn more". $label — the field label.
admin-ux-learn-more-field = sobre { $label }
admin-ux-propagation-hot = Entra em vigor em até 30 segundos em todos os processos do servidor.
admin-ux-propagation-restart = Entra em vigor depois que o servidor for reiniciado.
admin-ux-secret-set = Há um valor armazenado. Digite um novo para substituí-lo.
admin-ux-secret-unset = Nenhum valor armazenado.

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = Insira um valor.
admin-ux-err-number = Insira um número.
admin-ux-err-whole = Insira um número inteiro.
# $min, $max — the bounds the server declares for this field.
admin-ux-err-min = Insira { $min } ou mais.
admin-ux-err-max = Insira { $max } ou menos.
admin-ux-err-max-length = Use no máximo { $max } caracteres.
admin-ux-err-url = Insira um endereço completo começando com https:// ou http://.
admin-ux-err-email = Insira um endereço de e-mail, como name@example.org.
admin-ux-err-pattern = Este valor não tem o formato esperado.
admin-ux-err-option = Escolha uma das opções listadas.
admin-ux-err-out-of-range = O servidor recusou este valor porque ele está fora do intervalo permitido.
admin-ux-err-too-long = O servidor recusou este valor porque ele é longo demais.
admin-ux-err-invalid = O servidor recusou este valor. Verifique-o e salve novamente.
admin-ux-err-pinned = Este valor é definido no ambiente do servidor e não pode ser alterado aqui.
admin-ux-err-locked = Este valor está bloqueado em um escopo mais amplo e não pode ser alterado aqui.

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = Padrão
admin-ux-source-stored = Armazenado
# $name — the environment variable, e.g. MW_LOG_LEVEL.
admin-ux-source-env-pin = Definido por { $name }
admin-ux-source-env-fallback = Valor alternativo de { $name }
admin-ux-source-build = Definido na compilação
# $scope — "global" or a domain name.
admin-ux-source-locked = Bloqueado por { $scope }
admin-ux-source-read-only = Somente leitura
admin-ux-source-restart = Requer reinicialização
admin-ux-source-env-pin-reason = O ambiente do servidor define este valor com { $name }. Altere-o lá e reinicie o servidor.
admin-ux-source-build-reason = Este valor é fixado quando o servidor é compilado.
admin-ux-source-locked-reason = Um bloqueio no escopo { $scope } fixa este valor. Remova o bloqueio lá para alterá-lo.
admin-ux-source-read-only-reason = O console mostra este valor, mas não pode alterá-lo. A documentação informa onde ele é definido.

# -- History (UX8) ------------------------------------------------------------
# $who — an admin name; $time — date and time in the reader's locale.
admin-ux-history-changed-by-at = Alterado pela última vez por { $who } em { $time }
admin-ux-history-changed-by = Alterado pela última vez por { $who }
admin-ux-history-changed-at = Alterado pela última vez em { $time }
admin-ux-history-never = Não alterado pelo console
admin-ux-history-link = Histórico

# -- Revert to default (UX7) --------------------------------------------------
# $value — the default as shown, e.g. "587".
admin-ux-revert-default-is = Padrão: { $value }
admin-ux-revert = Restaurar o padrão
# Added for screen readers after "Revert to default". $label — the field label.
admin-ux-revert-field = ({ $label })
admin-ux-revert-done = Restaurado para o padrão ({ $value }).

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = { $who } alterou isto em { $time } enquanto você editava.
admin-ux-stale-by = { $who } alterou isto enquanto você editava.
admin-ux-stale-unknown = Outra pessoa alterou isto enquanto você editava.
admin-ux-stale-choice = Recarregue para ver a versão da outra pessoa e perder suas edições, ou mantenha as suas e salve-as por cima.
admin-ux-stale-reload = Recarregar
admin-ux-stale-keep = Manter as minhas

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = Verificando o que esta alteração afeta…
admin-ux-impact-heading = Esta alteração afeta
# $label — what is affected (e.g. "Users refused"); $count — how many.
admin-ux-impact-count = { $label }: { $count }
admin-ux-impact-more =
    { $count ->
        [one] e mais 1
       *[other] e mais { $count }
    }
admin-ux-impact-none = Nada mais muda.
admin-ux-impact-unavailable = Prévia de impacto indisponível — o servidor não informou o que esta alteração afeta.

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = Filtro
admin-ux-stream-pause = Pausar
admin-ux-stream-resume = Retomar
admin-ux-stream-clear = Limpar
admin-ux-stream-announce = Ler novas entradas em voz alta
admin-ux-stream-connecting = Conectando…
admin-ux-stream-live = Ao vivo
admin-ux-stream-ended = Encerrado
admin-ux-stream-lost = Conexão perdida
admin-ux-stream-reconnect = Reconectar
admin-ux-stream-paused =
    { $count ->
        [0] Pausado
        [one] Pausado — 1 nova entrada retida
       *[other] Pausado — { $count } novas entradas retidas
    }
admin-ux-stream-no-match = Nenhuma entrada corresponde ao filtro.

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = Campo

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = Aparência
admin-ux-appearance-title = Aparência do console
admin-ux-appearance-note = Estas opções alteram apenas o console de administração, neste navegador.
admin-ux-appearance-following = O console segue o tema do webmail deste navegador até que você escolha aqui. Estas opções alteram apenas o console de administração, neste navegador.
admin-ux-appearance-mode = Modo
admin-ux-appearance-mode-light = Claro
admin-ux-appearance-mode-dark = Escuro
admin-ux-appearance-mode-system = Seguir o sistema
admin-ux-appearance-theme = Tema
admin-ux-appearance-accent = Cor de destaque
admin-ux-appearance-light = Claro
admin-ux-appearance-dark = Escuro
admin-ux-appearance-reset = Usar o tema do webmail
