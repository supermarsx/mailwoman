# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-ux. Source: ../en/admin-ux.ftl

admin-ux-close = Fechar

# -- Page states (UX11) ---------------------------------------------------------
# $reason — a sentence saying why the data is not there.
admin-ux-state-unavailable = Indisponível — { $reason }
admin-ux-unavailable-default = este servidor ainda não oferece esta funcionalidade.
# $code — the server's error code or "HTTP <status>".
admin-ux-state-code = Código de erro: { $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = Abrir
admin-ux-list-back = Voltar à lista

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = Sem alterações por guardar
admin-ux-form-changes =
    { $count ->
        [one] 1 alteração por guardar
       *[other] { $count } alterações por guardar
    }
admin-ux-form-discard = Descartar alterações
admin-ux-form-saving = A guardar…
admin-ux-form-saved =
    { $count ->
        [one] 1 alteração guardada.
       *[other] { $count } alterações guardadas.
    }
admin-ux-form-fix =
    { $count ->
        [one] Nada foi guardado. Corrija o campo assinalado abaixo.
       *[other] Nada foi guardado. Corrija os { $count } campos assinalados abaixo.
    }
admin-ux-learn-more = Saber mais
# Added for screen readers after "Learn more". $label — the field label.
admin-ux-learn-more-field = sobre { $label }
admin-ux-propagation-hot = Aplica-se em 30 segundos em todos os processos do servidor.
admin-ux-propagation-restart = Aplica-se depois de o servidor reiniciar.
admin-ux-secret-set = Há um valor guardado. Escreva um novo para o substituir.
admin-ux-secret-unset = Não há nenhum valor guardado.

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = Introduza um valor.
admin-ux-err-number = Introduza um número.
admin-ux-err-whole = Introduza um número inteiro.
# $min, $max — the bounds the server declares for this field.
admin-ux-err-min = Introduza { $min } ou mais.
admin-ux-err-max = Introduza { $max } ou menos.
admin-ux-err-max-length = Utilize no máximo { $max } carateres.
admin-ux-err-url = Introduza um endereço completo que comece por https:// ou http://.
admin-ux-err-email = Introduza um endereço de e-mail, por exemplo nome@example.org.
admin-ux-err-pattern = Este valor não tem o formato esperado.
admin-ux-err-option = Escolha uma das opções da lista.
admin-ux-err-out-of-range = O servidor recusou este valor porque está fora do intervalo permitido.
admin-ux-err-too-long = O servidor recusou este valor porque é demasiado longo.
admin-ux-err-invalid = O servidor recusou este valor. Verifique-o e guarde novamente.
admin-ux-err-pinned = Este valor está definido no ambiente do servidor e não pode ser alterado aqui.
admin-ux-err-locked = Este valor está bloqueado num âmbito mais abrangente e não pode ser alterado aqui.

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = Predefinição
admin-ux-source-stored = Guardado
# $name — the environment variable, e.g. MW_LOG_LEVEL.
admin-ux-source-env-pin = Definido por { $name }
admin-ux-source-env-fallback = Alternativa de { $name }
admin-ux-source-build = Definido na compilação
# $scope — "global" or a domain name.
admin-ux-source-locked = Bloqueado por { $scope }
admin-ux-source-read-only = Só de leitura
admin-ux-source-restart = Reinício necessário
admin-ux-source-env-pin-reason = O ambiente do servidor define este valor com { $name }. Altere-o aí e reinicie o servidor.
admin-ux-source-build-reason = Este valor é fixado quando o servidor é compilado.
admin-ux-source-locked-reason = Um bloqueio no âmbito { $scope } fixa este valor. Remova aí o bloqueio para o alterar.
admin-ux-source-read-only-reason = A consola mostra este valor, mas não o pode alterar. A documentação indica onde é definido.

# -- History (UX8) ------------------------------------------------------------
# $who — an admin name; $time — date and time in the reader's locale.
admin-ux-history-changed-by-at = Última alteração por { $who } em { $time }
admin-ux-history-changed-by = Última alteração por { $who }
admin-ux-history-changed-at = Última alteração em { $time }
admin-ux-history-never = Não alterado a partir da consola
admin-ux-history-link = Histórico

# -- Revert to default (UX7) --------------------------------------------------
# $value — the default as shown, e.g. "587".
admin-ux-revert-default-is = Predefinição: { $value }
admin-ux-revert = Repor a predefinição
# Added for screen readers after "Revert to default". $label — the field label.
admin-ux-revert-field = ({ $label })
admin-ux-revert-done = Predefinição reposta ({ $value }).

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = { $who } alterou isto em { $time } enquanto editava.
admin-ux-stale-by = { $who } alterou isto enquanto editava.
admin-ux-stale-unknown = Outra pessoa alterou isto enquanto editava.
admin-ux-stale-choice = Recarregue para ver a versão da outra pessoa e perder as suas edições, ou mantenha as suas e guarde-as por cima.
admin-ux-stale-reload = Recarregar
admin-ux-stale-keep = Manter as minhas

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = A verificar o que esta alteração afeta…
admin-ux-impact-heading = Esta alteração afeta
# $label — what is affected (e.g. "Users refused"); $count — how many.
admin-ux-impact-count = { $label }: { $count }
admin-ux-impact-more =
    { $count ->
        [one] e mais 1
       *[other] e mais { $count }
    }
admin-ux-impact-none = Nada mais é alterado.
admin-ux-impact-unavailable = Pré-visualização do impacto indisponível — o servidor não indicou o que esta alteração afeta.

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = Filtrar
admin-ux-stream-pause = Pausar
admin-ux-stream-resume = Retomar
admin-ux-stream-clear = Limpar
admin-ux-stream-announce = Ler as novas entradas em voz alta
admin-ux-stream-connecting = A ligar…
admin-ux-stream-live = Em direto
admin-ux-stream-ended = Terminado
admin-ux-stream-lost = Ligação perdida
admin-ux-stream-reconnect = Voltar a ligar
admin-ux-stream-paused =
    { $count ->
        [0] Em pausa
        [one] Em pausa — 1 nova entrada retida
       *[other] Em pausa — { $count } novas entradas retidas
    }
admin-ux-stream-no-match = Nenhuma entrada corresponde ao filtro.

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = Campo

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = Aspeto
admin-ux-appearance-title = Aspeto da consola
admin-ux-appearance-note = Estas opções alteram apenas a consola de administração, neste navegador.
admin-ux-appearance-following = A consola segue o tema do webmail deste navegador até escolher aqui. Estas opções alteram apenas a consola de administração, neste navegador.
admin-ux-appearance-mode = Modo
admin-ux-appearance-mode-light = Claro
admin-ux-appearance-mode-dark = Escuro
admin-ux-appearance-mode-system = Igual ao sistema
admin-ux-appearance-theme = Tema
admin-ux-appearance-accent = Cor de destaque
admin-ux-appearance-light = Claro
admin-ux-appearance-dark = Escuro
admin-ux-appearance-reset = Usar o tema do webmail

# -- List editor (f33) ----------------------------------------------------------
admin-ux-list-add = Adicionar
admin-ux-list-remove = Remover { $item }
admin-ux-list-none = Ainda nenhum.
admin-ux-list-duplicate = { $item } já está na lista.
admin-ux-list-empty-entry = Escreva primeiro uma entrada.
