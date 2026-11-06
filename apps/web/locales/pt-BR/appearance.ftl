# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: appearance. Source: ../en/appearance.ftl

## Theme mode (fixed / system / schedule)

appearance-mode = Quando trocar
appearance-mode-fixed = Um tema
appearance-mode-system = Seguir o sistema
appearance-mode-schedule = Por horário do dia

appearance-mode-hint-fixed = O tema escolhido permanece até você alterá-lo.
appearance-mode-hint-system = Troca de acordo com a configuração de claro e escuro do seu sistema operacional.
appearance-mode-hint-schedule = Troca para o tema escuro durante o horário definido abaixo.

## Gallery

appearance-gallery = Tema
appearance-gallery-hint = Escolher um tema aqui troca para um tema fixo.
appearance-active = Em uso
appearance-light = Claro
appearance-dark = Escuro

## The light/dark pair used by the two automatic modes

appearance-pair = Temas a alternar
appearance-pair-light = Tema claro
appearance-pair-dark = Tema escuro

## Schedule window

appearance-schedule = Horário escuro
appearance-schedule-start = De
appearance-schedule-end = Até

## Server sync

appearance-sync = Salvamento
appearance-sync-idle = Não iniciado.
appearance-sync-loading = Verificando sua conta…
appearance-sync-synced = Salvo na sua conta. Outros dispositivos aplicam isto ao iniciar.
appearance-sync-local-only = Salvo neste dispositivo. Entre para salvar estas configurações na sua conta.
appearance-sync-error = Não foi possível acessar o servidor. Salvo neste dispositivo; será enviado novamente na sua próxima alteração.
appearance-sync-reset = Esquecer a aparência salva na minha conta
appearance-sync-reset-done = Removido da sua conta. Este dispositivo mantém o que está exibindo agora.
appearance-sync-deployment = O padrão deste servidor é { $theme }.

## Theme names and descriptions (theme/tokens.ts THEME_META)

appearance-theme-dark = Escuro
appearance-theme-dark-desc = Página quase preta, texto off-white, ações em azul, alertas em vermelho, marcadores em amarelo.
appearance-theme-light = Claro
appearance-theme-light-desc = Página off-white, texto preto, ações em azul, alertas em vermelho, marcadores em amarelo.
appearance-theme-bauhaus-black = Preto
appearance-theme-bauhaus-black-desc = Página preta, texto e controles brancos, ações em azul, alertas em vermelho, marcadores em amarelo.
appearance-theme-paper = Papel
appearance-theme-paper-desc = Página off-white quente, tinta preta, azul suave, vermelho e ocre.
appearance-theme-blueprint = Planta técnica
appearance-theme-blueprint-desc = Página azul-escura, texto e linhas brancos, ações em ciano, marcadores em amarelo.
appearance-theme-terminal = Terminal
appearance-theme-terminal-desc = Página quase preta, texto verde, marcadores âmbar. Apenas cores; a fonte não muda.
appearance-theme-primary = Blocos primários
appearance-theme-primary-desc = Página branca, texto preto, blocos chapados em azul, vermelho e amarelo.
appearance-theme-slate-light = Ardósia claro
appearance-theme-slate-light-desc = Superfícies cinza-azuladas frias sobre uma página tingida, destaque azul-marinho suave.
appearance-theme-slate-dark = Ardósia escuro
appearance-theme-slate-dark-desc = Tema escuro cinza-azulado frio com destaque azul suave.
appearance-theme-ocean-light = Oceano claro
appearance-theme-ocean-light-desc = Tema claro azul-esverdeado com destaque verde-azulado escuro.
appearance-theme-ocean-dark = Oceano escuro
appearance-theme-ocean-dark-desc = Tema escuro azul-esverdeado com destaque ciano.
appearance-theme-plum-light = Ameixa claro
appearance-theme-plum-light-desc = Tema claro roxo suave com destaque violeta.
appearance-theme-plum-dark = Ameixa escuro
appearance-theme-plum-dark-desc = Tema escuro roxo suave com destaque violeta.
appearance-theme-grove-light = Bosque claro
appearance-theme-grove-light-desc = Tons quentes de papel com textura de madeira e destaque verde-musgo.
appearance-theme-grove-dark = Bosque escuro
appearance-theme-grove-dark-desc = Tons quentes de nogueira com textura de madeira e destaque verde-musgo.
appearance-theme-amoled = AMOLED
appearance-theme-amoled-desc = Tema escuro em preto puro; pixels apagados em telas OLED.
appearance-theme-hc-light = Alto contraste (claro)
appearance-theme-hc-light-desc = Preto sobre branco com bordas pretas e sem superfícies tingidas.
appearance-theme-hc-dark = Alto contraste (escuro)
appearance-theme-hc-dark-desc = Branco sobre preto com bordas brancas e sem superfícies tingidas.

## Gallery groups (theme/registry.ts)

appearance-family-core = Padrão
appearance-family-bauhaus = Bauhaus
appearance-family-slate = Ardósia
appearance-family-amoled = AMOLED
appearance-family-contrast = Alto contraste

## Accents (theme/accentMath.ts)

appearance-accent-default = Padrão do tema
appearance-accent-red = Vermelho
appearance-accent-orange = Laranja
appearance-accent-yellow = Amarelo
appearance-accent-green = Verde
appearance-accent-teal = Verde-azulado
appearance-accent-blue = Azul
appearance-accent-indigo = Índigo
appearance-accent-violet = Violeta
appearance-accent-magenta = Magenta
appearance-accent-graphite = Grafite

# -- Machine translation (2026-11): added to match en ---------------------
appearance-family-naturals = Tons naturais
appearance-family-specialty = Temas especiais
appearance-custom-title = Personalização da aparência
appearance-custom-description = Salva apenas para este espaço de trabalho com login ativo. As políticas do administrador controlam quais personalizações podem ser executadas.
appearance-custom-loading = Carregando personalização…
appearance-custom-error = A personalização da aparência não está disponível.
appearance-custom-css-on = CSS personalizado ativado
appearance-custom-css-off = CSS personalizado desativado pelo administrador
appearance-custom-js-on = Scripts de aparência ativados
appearance-custom-js-off = Scripts de aparência desativados pelo administrador
appearance-custom-locked = Os bloqueios de aparência do administrador suspendem o CSS personalizado. Os scripts não podem alterar campos bloqueados.
appearance-custom-css-locked = Os bloqueios de aparência do administrador suspendem este CSS. A configuração salva continua disponível abaixo.
appearance-custom-js-locked = Este script não pode alterar campos de aparência bloqueados por um administrador.
appearance-custom-css = CSS restrito
appearance-custom-css-help = Até 16 KiB. Somente propriedades visuais e seletores simples; sem URLs, importações, atributos, posicionamento ou !important. Os seletores ficam dentro do seu espaço de trabalho.
appearance-custom-js = Script de aparência isolado
appearance-custom-js-help = Até 16 KiB e 500 ms. Chame appearance.set com um objeto contendo accent '#3366cc' ou density 'compact'. Campos aceitos: accent, density, font e mode (light ou dark). context fornece os booleanos dark e reducedMotion. As alterações são temporárias; não há acesso a e-mails, dados da conta, rede nem ao DOM do aplicativo.
appearance-custom-save = Salvar personalização
appearance-custom-reset = Redefinir personalização
appearance-custom-reload = Descartar rascunho e recarregar a personalização salva
appearance-custom-discard = Descartar este rascunho de personalização não salvo e recarregar?
appearance-custom-saved = Personalização salva.
appearance-custom-safe = Recuperação: adicione appearanceSafe=1 à consulta da página para abrir este espaço de trabalho sem CSS ou scripts personalizados.
appearance-custom-accent = Cor de destaque personalizada
appearance-custom-accent-reset = Usar o destaque nomeado ou o do tema
appearance-text-mode-label = Estilo do texto
appearance-text-mode-normal = Texto normal
appearance-text-mode-leet = Leetspeak (1337)
appearance-text-mode-help = Use leetspeak nos rótulos da interface e na leitura de mensagens. A edição e as mensagens enviadas mantêm o texto original.
appearance-text-mode-example = Olá, Mailwoman!
appearance-preference-appearance-fontSize = Tamanho da fonte da interface
appearance-preference-appearance-fontWeight = Peso da fonte da interface
appearance-preference-appearance-fontColor = Cor da fonte da interface
appearance-preference-appearance-lowContrast = Modo de baixo contraste
appearance-preference-mail-messageListStyle = Estilo da lista de mensagens
appearance-preference-mail-messageListPreview = Mostrar pré-visualizações do corpo das mensagens
appearance-preference-mail-afterDelete = Depois de excluir uma mensagem
appearance-preference-reader-attachmentsAtEnd = Mostrar anexos no final das mensagens
appearance-preference-ui-notifyNewVersion = Avisar quando houver uma nova versão do Mailwoman
appearance-preference-option-300 = Leve
appearance-preference-option-400 = Normal
appearance-preference-option-500 = Médio
appearance-preference-option-600 = Seminegrito
appearance-preference-option-cards = Cartões
appearance-preference-option-table = Linhas no estilo Gmail
appearance-preference-option-next = Abrir a próxima mensagem
appearance-preference-option-previous = Abrir a mensagem anterior
appearance-preference-option-close = Fechar a mensagem
appearance-preference-inherit = Usar o padrão do administrador ou do tema
appearance-preference-mail-messageListOrder = Ordem da lista de mensagens
appearance-preference-option-newest = Mais recentes primeiro
appearance-preference-option-oldest = Mais antigas primeiro
appearance-preference-option-sender = Remetente
appearance-preference-option-subject = Assunto
appearance-theme-cyberpunk = Cyberpunk
appearance-theme-cyberpunk-desc = Superfícies cor de carvão com ações em amarelo elétrico e destaques em ciano.
appearance-theme-retrowave = Retrowave
appearance-theme-retrowave-desc = Superfícies violeta profundo com ações em magenta e destaques neon em ciano.
appearance-theme-fight-club = Fight Club
appearance-theme-fight-club-desc = Superfícies verde-oliva desgastadas, texto cor de papel quente e ações rosa-sabonete.
appearance-accent-gold = Dourado
appearance-accent-brown = Marrom
appearance-accent-beige = Bege
appearance-accent-taupe = Taupe
appearance-accent-silver = Prata
appearance-accent-stone = Pedra
appearance-theme-slate-mist = Slate Mist
appearance-theme-slate-mist-desc = Tema claro cinza-azulado de tom médio com destaque verde-azulado.
appearance-theme-slate-midnight = Slate Midnight
appearance-theme-slate-midnight-desc = Tema escuro cinza-azulado quase preto com destaque azul-gelo.
appearance-theme-nineteen-eighty-four = 1984
appearance-theme-nineteen-eighty-four-desc = Concreto cinza de ministério, texto cor de papel desbotado, vermelho Grande Irmão; mensagens em fonte monoespaçada.
appearance-theme-pop-art = Pop Art
appearance-theme-pop-art-desc = Página branca, contornos pretos, vermelho, azul e amarelo primários, pontos Ben-Day na barra lateral.
