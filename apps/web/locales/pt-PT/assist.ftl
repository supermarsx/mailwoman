# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: assist. Source: ../en/assist.ftl
#
# HONESTY: the "what left the device" disclosure (assist-left-*, assist-disclosure-*)
# is a concrete, factual statement of exactly what the AI assistant can and cannot send.
# Keep "never sends …", "excluded by default", and "Send is never automated" accurate.

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = Assistente de IA
assist-heading = Assistente de IA
assist-transcript-label = Conversa com o assistente de IA
assist-proposed-action = Ação proposta
assist-outbox-note = Isto colocaria uma mensagem na sua Caixa de saída. Nada é enviado até o confirmar lá.
assist-review-outbox = Rever na Caixa de saída
assist-review = Rever
assist-empty = Peça ao assistente de IA para resumir, redigir ou encontrar coisas. Pode propor ações, mas é sempre o utilizador que as confirma.
assist-error = O assistente de IA não conseguiu responder neste momento.
assist-input-label = Mensagem para o assistente de IA
assist-input-placeholder = Pergunte ao assistente de IA…
assist-thinking = A pensar…
assist-ask = Perguntar

# -- Composer tools ---------------------------------------------------------
assist-composer-label = Ajuda na escrita
assist-composer-toolbar = Ferramentas do editor de mensagens
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
assist-composer-error = O assistente de IA não conseguiu concluir isso neste momento.
assist-suggested-edit = Edição sugerida
assist-apply-draft = Aplicar ao rascunho
assist-discard = Descartar

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = Etiquetas sugeridas
assist-apply-auto = Aplicar automaticamente
assist-apply-label = Aplicar { $label }
assist-remove-label = Remover { $label }
assist-apply = Aplicar
assist-undo = Anular

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = Manter premido para ditar
assist-dictate-stop = Parar ditado
assist-dictate-hold-text = Manter premido para ditar
assist-dictate-listening = ● A ouvir…
assist-dictate-endpoint-note = O áudio é transcrito pelo endpoint do seu assistente de IA.
assist-dictate-err = Erro no ditado.
assist-transcribe-err = Não foi possível transcrever o áudio.
assist-mic-err = Microfone indisponível.

# -- Semantic search --------------------------------------------------------
assist-semantic-label = Pesquisa semântica
assist-semantic-note = O texto da pesquisa é enviado para o endpoint do seu assistente de IA para ordenar os resultados por significado.

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = O que pode sair deste dispositivo
assist-left-1 = o texto da mensagem selecionada (assunto + corpo) para a capacidade escolhida
assist-left-2 = o anfitrião do endpoint para onde foi enviado
assist-left-3 = nunca: conteúdo encriptado ponto a ponto (excluído por predefinição)
assist-left-4 = nunca: anexos (excluídos por predefinição)
assist-left-5 = nunca: as suas credenciais ou outras contas
assist-disclosure-off = O assistente de IA está desativado. Nenhum conteúdo de mensagens sai deste dispositivo.
assist-disclosure-sentence = Quando utiliza uma ferramenta do assistente de IA, o texto da mensagem selecionada é encaminhado através de proxy para { $host }. { $excl } O envio nunca é automático — confirma sempre antes de qualquer coisa sair da sua Caixa de saída.
assist-disclosure-excl-one = Nunca envia { $a }.
assist-disclosure-excl-two = Nunca envia { $a } nem { $b }.
assist-disclosure-admin-allowed = O seu administrador permitiu o envio de conteúdo encriptado e de anexos.
assist-excluded-e2ee = conteúdo encriptado ponto a ponto
assist-excluded-attachments = anexos

# -- Admin → AI assistant (deployment configuration) ------------------------------
# Wording here states what the server does. "Stop" takes effect on the running
# server; every other change is read when the server starts.
assist-admin-title = Assistente de IA
assist-admin-intro = O assistente de IA envia o texto das mensagens que os utilizadores selecionam para o endpoint de IA configurado aqui. Não pode enviar, eliminar nem aceitar correio. O conteúdo encriptado ponto a ponto e os anexos são retidos, a menos que os permita abaixo.
assist-admin-load-unreadable = Não é possível ler a configuração guardada do assistente de IA: a respetiva chave de API não abre com a chave deste servidor (MW_SERVER_KEY). Foi guardada com outra chave ou foi alterada. O assistente de IA está desativado. Preencha o formulário (com a chave de API) e guarde para a substituir.
assist-admin-load-error = Não foi possível carregar a configuração do assistente de IA. A opção de guardar está desativada para que a configuração guardada não seja substituída.
assist-admin-status-running = O assistente de IA está em execução. Os pedidos são enviados para { $host }.
assist-admin-status-running-pending = O assistente de IA está em execução com a configuração com que o servidor foi iniciado. As alterações guardadas entram em vigor quando o servidor for reiniciado.
assist-admin-status-off = O assistente de IA está desativado. Todos os pedidos ao assistente de IA são recusados.
assist-admin-status-pending = O assistente de IA não está em execução. A configuração guardada entra em vigor quando o servidor for reiniciado.
assist-admin-status-local-program-not-allowed = O assistente de IA não está em execução. O programa configurado { $program } não consta de MW_ASSIST_LOCAL_PROGRAMS no servidor, pelo que o servidor não o iniciou.
assist-admin-status-no-endpoint = O assistente de IA está ativado, mas não está em execução porque não há nenhum endpoint utilizável configurado.
assist-admin-stop = Parar o assistente de IA agora
assist-admin-stop-note = A paragem entra em vigor imediatamente para todos os utilizadores e mantém-se após reinícios.
assist-admin-resume = Ativar o assistente de IA
assist-admin-resume-note = Se o servidor foi iniciado com o assistente de IA desativado, a ativação entra em vigor quando o servidor for reiniciado.
assist-admin-kill-error = Não foi possível alterar o estado de ativação do assistente de IA.
assist-admin-endpoint = Endpoint
assist-admin-endpoint-note = É o servidor que contacta este endpoint; os navegadores nunca o fazem. As alterações entram em vigor quando o servidor for reiniciado.
assist-admin-kind = Tipo de endpoint
assist-admin-kind-none = Nenhum
assist-admin-kind-open-ai-compatible = Compatível com OpenAI
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = Programa local
assist-admin-local-program-enable = Para que o assistente de IA execute um programa neste servidor, defina MW_ASSIST_LOCAL_PROGRAMS com o caminho absoluto do programa no local onde o servidor é iniciado (separe vários caminhos com vírgulas) e reinicie o servidor.
assist-admin-local-program-note = O servidor inicia este programa para cada pedido ao assistente de IA e passa-lhe o pedido pela entrada padrão, com o texto da mensagem que o utilizador selecionou. Lê a resposta a partir da saída padrão do programa. Só podem ser escolhidos programas que constem de MW_ASSIST_LOCAL_PROGRAMS no servidor.
assist-admin-local-program = Programa
assist-admin-local-program-unlisted = { $program } (não listado)
assist-admin-local-program-unlisted-note = Este programa não consta de MW_ASSIST_LOCAL_PROGRAMS, pelo que o servidor não o executa e não o aceitará quando guardar. Escolha um programa listado ou outro tipo de endpoint.
assist-admin-local-program-required = Escolha o programa a executar.
assist-admin-local-args = Argumentos, um por linha
assist-admin-local-args-note = Cada linha é passada ao programa como um argumento. Não é utilizada nenhuma shell.
assist-admin-base-url = URL base
assist-admin-base-url-required = Introduza o URL base do endpoint, a começar por http:// ou https://.
assist-admin-api-key = Chave de API
assist-admin-api-key-stored = Há uma chave guardada. O servidor não a volta a mostrar.
assist-admin-api-key-replace = Substituir a chave de API
assist-admin-api-key-replace-placeholder = Deixe vazio para manter a chave guardada
assist-admin-api-key-remove = Remover a chave guardada
assist-admin-api-key-keep = Manter a chave guardada
assist-admin-api-key-remove-pending = A chave guardada será removida quando guardar.
assist-admin-chat-model = Modelo de conversa
assist-admin-embed-model = Modelo de embeddings
assist-admin-stt-model = Modelo de voz para texto
assist-admin-model = Modelo
assist-admin-anthropic-version = Versão da API
assist-admin-max-tokens = Máximo de tokens por resposta
assist-admin-default-placeholder = Predefinição do servidor
assist-admin-grants = Capacidades
assist-admin-grants-note = Só podem ser utilizadas as capacidades assinaladas aqui.
assist-admin-cap-summarize = Resumir
assist-admin-cap-draft = Redigir e reescrever
assist-admin-cap-grammar = Gramática
assist-admin-cap-dictation = Ditado
assist-admin-cap-search-semantic = Pesquisa semântica
assist-admin-cap-auto-tag = Etiquetagem automática
assist-admin-cap-recap = Recapitulação
assist-admin-cap-assistant = Conversa com o assistente de IA
assist-admin-ceilings = Limites de dados
assist-admin-ceilings-note = Estes limites aplicam-se a todos os pedidos, seja qual for a capacidade.
assist-admin-accounts = Ids das contas cujo correio pode ser enviado, um por linha
assist-admin-accounts-note = Se não houver nenhuma conta listada, nenhum conteúdo de correio é enviado para o endpoint.
assist-admin-folders = Ids das pastas a que limitar o envio, um por linha
assist-admin-folders-note = Se não houver nenhuma pasta listada, todas as pastas das contas listadas são permitidas.
assist-admin-allow-e2ee = Permitir o envio de conteúdo encriptado ponto a ponto
assist-admin-allow-attachments = Permitir o envio de anexos
assist-admin-save = Guardar configuração
assist-admin-saved = Guardado.
assist-admin-saved-pending = Guardado. A alteração entra em vigor quando o servidor for reiniciado.
assist-admin-save-error = A configuração não foi guardada.
assist-admin-save-not-allowed = A configuração não foi guardada porque o servidor não permite este programa. Só podem ser utilizados programas que constem de MW_ASSIST_LOCAL_PROGRAMS.

# Rebuilt on the settings form (f33, t32-cfg §4.12).
assist-admin-unavailable = este servidor não disponibiliza as definições do assistente de IA.
assist-admin-off = Desativado
assist-admin-kind-help = Para onde o servidor envia os pedidos ao assistente de IA; «Nenhum» desativa o endpoint sem parar o assistente.
assist-admin-kind-help-no-local = Para onde o servidor envia os pedidos ao assistente de IA. Pode ser escolhido um programa local assim que MW_ASSIST_LOCAL_PROGRAMS indicar um no local onde o servidor é iniciado.
assist-admin-base-url-help = O endereço da API do endpoint; obrigatório para um endpoint compatível com OpenAI, opcional para Anthropic.
assist-admin-api-key-help = Só de escrita: o servidor guarda-a selada e nunca a volta a mostrar; alterar o tipo ou o URL base elimina uma chave guardada.
assist-admin-chat-model-help = O modelo utilizado para resumos, rascunhos e conversa; vazio utiliza a predefinição do servidor.
assist-admin-embed-model-help = O modelo utilizado para a pesquisa semântica; vazio utiliza a predefinição do servidor.
assist-admin-stt-model-help = O modelo utilizado para o ditado; vazio utiliza a predefinição do servidor.
assist-admin-model-help = O modelo da Anthropic; vazio utiliza a predefinição do servidor.
assist-admin-anthropic-version-help = O cabeçalho anthropic-version enviado com cada pedido; vazio utiliza a predefinição do servidor.
assist-admin-max-tokens-help = A resposta mais longa pedida; vazio utiliza a predefinição do servidor.
assist-admin-local-args-label = Argumentos
assist-admin-accounts-label = Ids das contas cujo correio pode ser enviado
assist-admin-accounts-none = Nenhuma conta: não é enviado nenhum conteúdo de correio.
assist-admin-folders-label = Ids das pastas a que limitar o envio
assist-admin-folders-none = Nenhuma pasta: todas as pastas das contas listadas.
assist-admin-allow-e2ee-help = Desativado retém o conteúdo encriptado ponto a ponto em todos os pedidos.
assist-admin-allow-attachments-help = Desativado retém os anexos em todos os pedidos.
assist-admin-rate-heading = Limite de pedidos
assist-admin-rate-group-help = Pedidos por minuto que o servidor aceita para o assistente de IA, no total de todos os utilizadores.
assist-admin-rate-unavailable = Este servidor ainda não disponibiliza o limite de pedidos na consola; é definido com MW_ASSIST_RATE_LIMIT_PER_MIN no local onde o servidor é iniciado.
assist-admin-rate-limit = Pedidos por minuto
assist-admin-rate-limit-help = 0 significa sem limite; no máximo 10000.
assist-admin-rate-none = 0 (sem limite)
assist-admin-stop-title = Parar o assistente de IA para todos?
assist-admin-stop-body = A partir de agora, todos os pedidos ao assistente de IA são recusados, para todos os utilizadores, até voltar a ser ativado. A configuração é mantida.
assist-admin-stopped-done = O assistente de IA está parado.
assist-admin-resumed-done = O assistente de IA está ativado.
