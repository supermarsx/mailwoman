# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: assist. Source: ../en/assist.ftl

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = Assistente de IA
assist-heading = Assistente de IA
assist-transcript-label = Conversa com o assistente de IA
assist-proposed-action = Ação proposta
assist-outbox-note = Isto colocaria uma mensagem na sua Caixa de saída. Nada é enviado até você confirmar lá.
assist-review-outbox = Revisar na Caixa de saída
assist-review = Revisar
assist-empty = Peça ao assistente de IA para resumir, redigir ou encontrar coisas. Ele pode propor ações, mas você sempre as confirma.
assist-error = O assistente de IA não conseguiu responder agora.
assist-input-label = Enviar mensagem ao assistente de IA
assist-input-placeholder = Pergunte ao assistente de IA…
assist-thinking = Pensando…
assist-ask = Perguntar

# -- Composer tools ---------------------------------------------------------
assist-composer-label = Ajuda na escrita
assist-composer-toolbar = Ferramentas do editor
assist-tool-grammar = Corrigir gramática
assist-tool-rewrite = Reescrever
assist-tool-tone = Ajustar tom
assist-tool-translate = Traduzir
assist-toolarg-tone = Tom
assist-toolarg-translate = Idioma
assist-tone-neutral = Neutro
assist-tone-friendly = Amigável
assist-tone-formal = Formal
assist-tone-concise = Conciso
assist-tone-assertive = Assertivo
assist-busy = { $label }…
assist-composer-empty-error = Escreva algo primeiro.
assist-composer-error = O assistente de IA não conseguiu concluir isso agora.
assist-suggested-edit = Edição sugerida
assist-apply-draft = Aplicar ao rascunho
assist-discard = Descartar

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = Marcadores sugeridos
assist-apply-auto = Aplicar automaticamente
assist-apply-label = Aplicar { $label }
assist-remove-label = Remover { $label }
assist-apply = Aplicar
assist-undo = Desfazer

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = Segure para ditar
assist-dictate-stop = Parar ditado
assist-dictate-hold-text = 🎙 Segure para ditar
assist-dictate-listening = ● Ouvindo…
assist-dictate-endpoint-note = O áudio é transcrito pelo endpoint do seu assistente de IA.
assist-dictate-err = Erro no ditado.
assist-transcribe-err = Não foi possível transcrever o áudio.
assist-mic-err = Microfone indisponível.

# -- Semantic search --------------------------------------------------------
assist-semantic-label = Pesquisa semântica
assist-semantic-note = O texto da consulta é enviado ao endpoint do seu assistente de IA para ordenar por significado.

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = O que pode sair deste dispositivo
assist-left-1 = o texto da mensagem selecionada (assunto + corpo) para o recurso escolhido
assist-left-2 = o host do endpoint para o qual foi enviado
assist-left-3 = nunca: conteúdo com criptografia de ponta a ponta (excluído por padrão)
assist-left-4 = nunca: anexos (excluídos por padrão)
assist-left-5 = nunca: suas credenciais ou outras contas
assist-disclosure-off = O assistente de IA está desativado. Nenhum conteúdo de mensagem sai deste dispositivo.
assist-disclosure-sentence = Quando você usa uma ferramenta do assistente de IA, o texto da mensagem selecionada é encaminhado para { $host }. { $excl } O envio nunca é automatizado — você sempre confirma antes que algo saia da sua Caixa de saída.
assist-disclosure-excl-one = Ele nunca envia { $a }.
assist-disclosure-excl-two = Ele nunca envia { $a } nem { $b }.
assist-disclosure-admin-allowed = Seu administrador permitiu o envio de conteúdo criptografado e de anexos.
assist-excluded-e2ee = conteúdo com criptografia de ponta a ponta
assist-excluded-attachments = anexos

# -- Admin → AI assistant (deployment configuration) ------------------------------
# Wording here states what the server does. "Stop" takes effect on the running
# server; every other change is read when the server starts.
assist-admin-title = Assistente de IA
assist-admin-intro = O assistente de IA envia o texto de mensagens que os usuários selecionam para o endpoint de IA configurado aqui. Ele não pode enviar, excluir nem aceitar e-mails. Conteúdo com criptografia de ponta a ponta e anexos são retidos, a menos que você os permita abaixo.
assist-admin-load-unreadable = A configuração armazenada do assistente de IA não pode ser lida: a chave de API dela não abre com a chave deste servidor (MW_SERVER_KEY). Ela foi salva com outra chave ou foi alterada. O assistente de IA está desativado. Preencha o formulário, incluindo a chave de API, e salve para substituí-la.
assist-admin-load-error = Não foi possível carregar a configuração do assistente de IA. O salvamento está desativado para que a configuração armazenada não seja sobrescrita.
assist-admin-status-running = O assistente de IA está em execução. As solicitações vão para { $host }.
assist-admin-status-running-pending = O assistente de IA está em execução com a configuração com que o servidor iniciou. As alterações salvas entram em vigor quando o servidor reiniciar.
assist-admin-status-off = O assistente de IA está desativado. Todas as solicitações ao assistente de IA são recusadas.
assist-admin-status-pending = O assistente de IA não está em execução. A configuração salva entra em vigor quando o servidor reiniciar.
assist-admin-status-local-program-not-allowed = O assistente de IA não está em execução. O programa configurado { $program } não está listado em MW_ASSIST_LOCAL_PROGRAMS no servidor, então o servidor não o iniciou.
assist-admin-status-no-endpoint = O assistente de IA está ativado, mas não está em execução porque nenhum endpoint utilizável está configurado.
assist-admin-stop = Parar o assistente de IA agora
assist-admin-stop-note = A parada entra em vigor imediatamente para todos os usuários e é mantida entre reinicializações.
assist-admin-resume = Ativar o assistente de IA
assist-admin-resume-note = Se o servidor iniciou com o assistente de IA desativado, ativá-lo entra em vigor quando o servidor reiniciar.
assist-admin-kill-error = Não foi possível alterar se o assistente de IA está ativado.
assist-admin-endpoint = Endpoint
assist-admin-endpoint-note = O servidor contata este endpoint; os navegadores nunca o fazem. As alterações entram em vigor quando o servidor reiniciar.
assist-admin-kind = Tipo de endpoint
assist-admin-kind-none = Nenhum
assist-admin-kind-open-ai-compatible = Compatível com OpenAI
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = Programa local
assist-admin-local-program-enable = Para que o assistente de IA execute um programa neste servidor, defina MW_ASSIST_LOCAL_PROGRAMS com o caminho absoluto do programa no ambiente onde o servidor é iniciado (separe vários caminhos com vírgulas) e reinicie o servidor.
assist-admin-local-program-note = O servidor inicia este programa a cada solicitação ao assistente de IA e passa a solicitação para ele pela entrada padrão, incluindo o texto da mensagem que o usuário selecionou. Ele lê a resposta da saída padrão do programa. Só podem ser escolhidos programas listados em MW_ASSIST_LOCAL_PROGRAMS no servidor.
assist-admin-local-program = Programa
assist-admin-local-program-unlisted = { $program } (não listado)
assist-admin-local-program-unlisted-note = Este programa não está listado em MW_ASSIST_LOCAL_PROGRAMS, então o servidor não o executa e não o aceitará quando você salvar. Escolha um programa listado ou outro tipo de endpoint.
assist-admin-local-program-required = Escolha o programa a executar.
assist-admin-local-args = Argumentos, um por linha
assist-admin-local-args-note = Cada linha é passada ao programa como um argumento. Nenhum shell é usado.
assist-admin-base-url = URL base
assist-admin-base-url-required = Informe a URL base do endpoint, começando com http:// ou https://.
assist-admin-api-key = Chave de API
assist-admin-api-key-stored = Há uma chave armazenada. O servidor não a mostra novamente.
assist-admin-api-key-replace = Substituir a chave de API
assist-admin-api-key-replace-placeholder = Deixe vazio para manter a chave armazenada
assist-admin-api-key-remove = Remover a chave armazenada
assist-admin-api-key-keep = Manter a chave armazenada
assist-admin-api-key-remove-pending = A chave armazenada será removida quando você salvar.
assist-admin-chat-model = Modelo de chat
assist-admin-embed-model = Modelo de embeddings
assist-admin-stt-model = Modelo de fala para texto
assist-admin-model = Modelo
assist-admin-anthropic-version = Versão da API
assist-admin-max-tokens = Máximo de tokens na resposta
assist-admin-default-placeholder = Padrão do servidor
assist-admin-grants = Recursos
assist-admin-grants-note = Somente os recursos marcados aqui podem ser usados.
assist-admin-cap-summarize = Resumir
assist-admin-cap-draft = Redigir e reescrever
assist-admin-cap-grammar = Gramática
assist-admin-cap-dictation = Ditado
assist-admin-cap-search-semantic = Pesquisa semântica
assist-admin-cap-auto-tag = Marcação automática
assist-admin-cap-recap = Recapitulação
assist-admin-cap-assistant = Chat com o assistente de IA
assist-admin-ceilings = Limites de dados
assist-admin-ceilings-note = Estes limites se aplicam a todas as solicitações, qualquer que seja o recurso.
assist-admin-accounts = IDs das contas cujos e-mails podem ser enviados, um por linha
assist-admin-accounts-note = Sem nenhuma conta listada, nenhum conteúdo de e-mail é enviado ao endpoint.
assist-admin-folders = IDs das pastas às quais limitar o envio, um por linha
assist-admin-folders-note = Sem nenhuma pasta listada, todas as pastas das contas listadas são permitidas.
assist-admin-allow-e2ee = Permitir o envio de conteúdo com criptografia de ponta a ponta
assist-admin-allow-attachments = Permitir o envio de anexos
assist-admin-save = Salvar configuração
assist-admin-saved = Salvo.
assist-admin-saved-pending = Salvo. A alteração entra em vigor quando o servidor reiniciar.
assist-admin-save-error = A configuração não foi salva.
assist-admin-save-not-allowed = A configuração não foi salva porque o servidor não permite este programa. Só podem ser usados programas listados em MW_ASSIST_LOCAL_PROGRAMS.
