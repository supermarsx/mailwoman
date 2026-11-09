# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: avatars. Source: ../en/avatars.ftl


# -- Machine translation (2026-11): added to match en ---------------------
avatars-title = Pessoas e imagens de contatos
avatars-mode = Imagens de remetentes e contatos
avatars-off = Ocultar imagens
avatars-initials = Gerar iniciais localmente
avatars-contacts = Usar imagens de contatos locais, com iniciais como alternativa
avatars-help = As iniciais são geradas neste dispositivo. As imagens de contatos usam apenas fotos PNG ou JPEG incorporadas já salvas nas suas próprias agendas de endereços, de até 512 × 512 pixels e 128 KiB. URLs de imagens remotas e serviços de avatar externos nunca são consultados. Esta preferência é salva com a sua conta e pode ser fixada pelo administrador.
avatars-default = Restaurar preferência herdada
avatars-sender-icons = Mostrar ícones incluídos de domínios de remetentes
avatars-sender-icons-help = Domínios reconhecidos de redes sociais, desenvolvimento, IA, compras e governo usam ícones incluídos neste aplicativo. Suas próprias imagens de contatos têm prioridade. Remetentes desconhecidos usam iniciais geradas localmente com 192 cores. Os ícones são indícios do domínio, não prova de que uma mensagem é autêntica. Nenhum serviço de logotipos externo é consultado.
avatars-icons-default = Restaurar preferência herdada de ícones
avatars-brand-avatars = Mostrar logotipos de marcas ao lado de remetentes verificados
avatars-brand-avatars-help = No leitor, uma empresa reconhecida mostra seu logotipo ao lado do remetente somente quando a mensagem passou no DKIM e em uma verificação DMARC alinhada para o domínio dessa empresa. Caso contrário, o avatar normal é usado. Os logotipos são incluídos neste aplicativo; nada é solicitado ao remetente.
avatars-brand-avatars-default = Restaurar preferência herdada de logotipos de marcas
avatars-domain-icon = Ícone do domínio { $name } · a identidade do remetente não é verificada por este ícone
avatars-favorites-first = Mostrar contatos favoritos primeiro nas sugestões de destinatários
avatars-favorites-first-help = Os contatos marcados com estrela aparecem antes das outras correspondências ao digitar um destinatário. Só a ordem muda; nenhuma sugestão é ocultada.
avatars-favorites-first-default = Restaurar preferência herdada de favoritos

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
avatars-require-auth = Autenticação necessária para logotipos de marcas

avatars-require-auth-always = Sempre (exige uma verificação aprovada)

avatars-require-auth-default = Restaurar preferência de autenticação herdada

avatars-require-auth-help = Escolha quando o logotipo de uma empresa reconhecida pode substituir as iniciais ao lado do remetente. Nunca: mostre sempre que o domínio do remetente corresponder. Somente se uma verificação falhar (padrão): mostre-a, a menos que a mensagem tenha falhado DKIM ou DMARC. Sempre: mostre apenas quando a mensagem passou DKIM e uma verificação DMARC alinhada para o domínio da própria empresa, para que o e-mail ainda não verificado mostre as iniciais. A correspondência de domínio inclui subdomínios como mail.example.com. Esta preferência é salva com sua conta e pode ser mantida pelo seu administrador.

avatars-require-auth-known-failure = Somente se uma verificação falhar

avatars-require-auth-never = Nunca (correspondência apenas no domínio)

avatars-unverified-sender = Remetente não verificado · nenhum ícone de marca é mostrado porque esta mensagem não passou na verificação de autenticação
