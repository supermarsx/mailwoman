# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: appearance. Source: ../en/appearance.ftl

## Theme mode (fixed / system / schedule)

appearance-mode = Quando mudar
appearance-mode-fixed = Um só tema
appearance-mode-system = Seguir o sistema
appearance-mode-schedule = Conforme a hora do dia

appearance-mode-hint-fixed = O tema escolhido mantém-se até o alterar.
appearance-mode-hint-system = Muda de acordo com a definição de claro e escuro do sistema operativo.
appearance-mode-hint-schedule = Muda para o tema escuro durante as horas definidas abaixo.

## Gallery

appearance-gallery = Tema
appearance-gallery-hint = Escolher aqui um tema muda para um tema fixo.
appearance-active = Em uso
appearance-light = Claro
appearance-dark = Escuro

## The light/dark pair used by the two automatic modes

appearance-pair = Temas entre os quais mudar
appearance-pair-light = Tema claro
appearance-pair-dark = Tema escuro

## Schedule window

appearance-schedule = Horas escuras
appearance-schedule-start = De
appearance-schedule-end = Até

## Server sync

appearance-sync = Gravação
appearance-sync-idle = Não iniciado.
appearance-sync-loading = A verificar a sua conta…
appearance-sync-synced = Guardado na sua conta. Os outros dispositivos aplicam-no quando arrancam.
appearance-sync-local-only = Guardado neste dispositivo. Inicie sessão para guardar estas definições na sua conta.
appearance-sync-error = Não foi possível contactar o servidor. Guardado neste dispositivo; será enviado novamente na próxima alteração.
appearance-sync-reset = Esquecer o aspeto guardado na minha conta
appearance-sync-reset-done = Removido da sua conta. Este dispositivo mantém o que está a mostrar agora.
appearance-sync-deployment = A predefinição deste servidor é { $theme }.

## Theme names and descriptions (theme/tokens.ts THEME_META)

appearance-theme-dark = Escuro
appearance-theme-dark-desc = Página quase preta, texto branco-sujo, ações a azul, alertas a vermelho, marcadores a amarelo.
appearance-theme-light = Claro
appearance-theme-light-desc = Página branco-sujo, texto preto, ações a azul, alertas a vermelho, marcadores a amarelo.
appearance-theme-bauhaus-black = Preto
appearance-theme-bauhaus-black-desc = Página preta, texto e controlos brancos, ações a azul, alertas a vermelho, marcadores a amarelo.
appearance-theme-paper = Papel
appearance-theme-paper-desc = Página branco-sujo quente, tinta preta, azul, vermelho e ocre suaves.
appearance-theme-blueprint = Planta técnica
appearance-theme-blueprint-desc = Página azul-escura, texto e linhas brancos, ações a ciano, marcadores a amarelo.
appearance-theme-terminal = Terminal
appearance-theme-terminal-desc = Página quase preta, texto verde, marcadores âmbar. Apenas as cores; o tipo de letra não muda.
appearance-theme-primary = Blocos primários
appearance-theme-primary-desc = Página branca, texto preto, blocos lisos azuis, vermelhos e amarelos.
appearance-theme-slate-light = Ardósia claro
appearance-theme-slate-light-desc = Superfícies azul-acinzentadas frias numa página tingida, cor de destaque azul-marinho suave.
appearance-theme-slate-dark = Ardósia escuro
appearance-theme-slate-dark-desc = Tema escuro azul-acinzentado frio com uma cor de destaque azul suave.
appearance-theme-ocean-light = Oceano claro
appearance-theme-ocean-light-desc = Tema claro verde-azulado com uma cor de destaque azul-petróleo escuro.
appearance-theme-ocean-dark = Oceano escuro
appearance-theme-ocean-dark-desc = Tema escuro verde-azulado com uma cor de destaque ciano.
appearance-theme-plum-light = Ameixa claro
appearance-theme-plum-light-desc = Tema claro púrpura suave com uma cor de destaque violeta.
appearance-theme-plum-dark = Ameixa escuro
appearance-theme-plum-dark-desc = Tema escuro púrpura suave com uma cor de destaque violeta.
appearance-theme-grove-light = Bosque claro
appearance-theme-grove-light-desc = Tons quentes de papel com textura de madeira e uma cor de destaque verde-musgo.
appearance-theme-grove-dark = Bosque escuro
appearance-theme-grove-dark-desc = Tons quentes de nogueira com textura de madeira e uma cor de destaque verde-musgo.
appearance-theme-amoled = AMOLED
appearance-theme-amoled-desc = Tema escuro preto puro; píxeis apagados nos ecrãs OLED.
appearance-theme-hc-light = Alto contraste (claro)
appearance-theme-hc-light-desc = Preto sobre branco com contornos pretos e sem superfícies tingidas.
appearance-theme-hc-dark = Alto contraste (escuro)
appearance-theme-hc-dark-desc = Branco sobre preto com contornos brancos e sem superfícies tingidas.

## Gallery groups (theme/registry.ts)

appearance-family-core = Predefinição
appearance-family-bauhaus = Bauhaus
appearance-family-naturals = Naturais
appearance-family-specialty = Temas especiais
appearance-family-slate = Ardósia
appearance-family-amoled = AMOLED
appearance-family-contrast = Alto contraste

## Accents (theme/accentMath.ts)

appearance-accent-default = Predefinição do tema
appearance-accent-red = Vermelho
appearance-accent-orange = Laranja
appearance-accent-yellow = Amarelo
appearance-accent-green = Verde
appearance-accent-teal = Azul-petróleo
appearance-accent-blue = Azul
appearance-accent-indigo = Índigo
appearance-accent-violet = Violeta
appearance-accent-magenta = Magenta
appearance-accent-graphite = Grafite

appearance-custom-title = Personalização pessoal do aspeto
appearance-custom-description = Guardada apenas para esta área de trabalho com sessão iniciada. As políticas do administrador controlam que personalizações podem ser executadas.
appearance-custom-loading = A carregar a personalização…
appearance-custom-error = A personalização do aspeto não está disponível.
appearance-custom-css-on = CSS personalizado ativado
appearance-custom-css-off = CSS personalizado desativado pelo administrador
appearance-custom-js-on = Scripts de aspeto ativados
appearance-custom-js-off = Scripts de aspeto desativados pelo administrador
appearance-custom-locked = Os bloqueios de aspeto do administrador suspendem o CSS personalizado. Os scripts não podem alterar campos bloqueados.
appearance-custom-css-locked = Os bloqueios de aspeto do administrador suspendem este CSS. A configuração guardada continua disponível abaixo.
appearance-custom-js-locked = Este script não pode alterar campos de aspeto bloqueados por um administrador.
appearance-custom-css = CSS restrito
appearance-custom-css-help = Até 16 KiB. Apenas propriedades visuais e seletores simples; sem URLs, importações, atributos, posicionamento nem !important. Os seletores ficam limitados à sua área de trabalho.
appearance-custom-js = Script de aspeto isolado
appearance-custom-js-help = Até 16 KiB e 500 ms. Chame appearance.set com um objeto que contenha accent '#3366cc' ou density 'compact'. Campos suportados: accent, density, font e mode (light ou dark). context fornece os booleanos dark e reducedMotion. As alterações são temporárias; não há acesso ao correio, aos dados da conta, à rede nem ao DOM da aplicação.
appearance-custom-save = Guardar personalização
appearance-custom-reset = Repor personalização
appearance-custom-reload = Descartar rascunho e recarregar a personalização guardada
appearance-custom-discard = Descartar este rascunho de personalização não guardado e recarregar?
appearance-custom-saved = Personalização guardada.
appearance-custom-safe = Recuperação: adicione appearanceSafe=1 à query da página para abrir esta área de trabalho sem CSS nem scripts personalizados.

appearance-custom-accent = Cor de destaque personalizada
appearance-custom-accent-reset = Usar a cor de destaque nomeada ou do tema

appearance-text-mode-label = Estilo do texto
appearance-text-mode-normal = Texto normal
appearance-text-mode-leet = Leetspeak (1337)
appearance-text-mode-help = Use leetspeak nos textos da interface e na leitura das mensagens. A edição e as mensagens enviadas mantêm o texto original.
appearance-text-mode-example = Olá, Mailwoman!

appearance-preference-appearance-fontSize = Tamanho do texto da interface
appearance-preference-appearance-fontWeight = Espessura do texto da interface
appearance-preference-appearance-fontColor = Cor do texto da interface
appearance-preference-appearance-lowContrast = Modo de baixo contraste
appearance-preference-mail-messageListStyle = Estilo da lista de mensagens
appearance-preference-mail-messageListPreview = Mostrar pré-visualização do corpo das mensagens
appearance-preference-mail-messageListOrder = Ordenação da lista de mensagens
appearance-preference-mail-list-pageSize = Mensagens carregadas por página (20 a 200)
appearance-preference-mail-afterDelete = Depois de eliminar uma mensagem
appearance-preference-reader-attachmentsAtEnd = Mostrar anexos no fim das mensagens
appearance-preference-ui-notifyNewVersion = Avisar-me quando houver uma nova versão do Mailwoman
appearance-preference-option-300 = Leve
appearance-preference-option-400 = Normal
appearance-preference-option-500 = Média
appearance-preference-option-600 = Seminegrito
appearance-preference-option-cards = Cartões
appearance-preference-option-table = Linhas ao estilo Gmail
appearance-preference-option-next = Abrir a mensagem seguinte
appearance-preference-option-previous = Abrir a mensagem anterior
appearance-preference-option-close = Fechar a mensagem
appearance-preference-option-newest = Mais recentes primeiro
appearance-preference-option-oldest = Mais antigas primeiro
appearance-preference-option-sender = Remetente
appearance-preference-option-subject = Assunto
appearance-preference-inherit = Usar predefinição do administrador ou do tema

appearance-theme-cyberpunk = Cyberpunk
appearance-theme-cyberpunk-desc = Superfícies em carvão com ações em amarelo elétrico e destaques em ciano.
appearance-theme-retrowave = Retrowave
appearance-theme-retrowave-desc = Superfícies em violeta profundo com ações em magenta e destaques néon em ciano.
appearance-theme-fight-club = Fight Club
appearance-theme-fight-club-desc = Superfícies em verde-azeitona gasto, texto em tons de papel e ações em rosa-sabão.
appearance-accent-gold = Ouro
appearance-accent-brown = Castanho
appearance-accent-beige = Bege
appearance-accent-taupe = Taupe
appearance-accent-silver = Prata
appearance-accent-stone = Pedra

appearance-theme-slate-mist = Ardósia névoa
appearance-theme-slate-mist-desc = Tema claro azul-acinzentado de tom médio com uma cor de destaque azul-petróleo.
appearance-theme-slate-midnight = Ardósia meia-noite
appearance-theme-slate-midnight-desc = Tema escuro azul-acinzentado quase preto com uma cor de destaque azul-gelo.
appearance-theme-nineteen-eighty-four = 1984
appearance-theme-nineteen-eighty-four-desc = Betão cinzento de ministério, texto em papel desbotado e vermelho do Grande Irmão; mensagens em monoespaçado.
appearance-theme-pop-art = Pop Art
appearance-theme-pop-art-desc = Página branca, contornos pretos, vermelho, azul e amarelo primários e pontos Ben-Day na barra lateral.

appearance-preference-mail-message-list-group = Agrupamento de mensagens

appearance-preference-option-sender-desc = Remetente, Z a A

appearance-preference-option-subject-desc = Assunto, Z a A

appearance-preference-option-sent-newest = Envio, mais recentes primeiro

appearance-preference-option-sent-oldest = Envio, mais antigas primeiro

appearance-preference-option-importance = Importância, alta primeiro

appearance-preference-option-importance-asc = Importância, baixa primeiro

appearance-preference-option-size = Tamanho, maiores primeiro

appearance-preference-option-size-asc = Tamanho, menores primeiro

appearance-preference-option-received = Data de receção

appearance-preference-option-sent = Data de envio
