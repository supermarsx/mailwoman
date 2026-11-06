# Brand logos

The web app shows a logo beside the name of third-party providers and apps: single sign-on providers on the sign-in page and in Admin > Single sign-on, AI assistant providers, analytics providers, webhook targets, egress destinations and the provider of a linked mailbox.

## Where the marks come from

- Path data comes from [Simple Icons](https://github.com/simple-icons/simple-icons), released under CC0-1.0 (public domain dedication). `apps/web/scripts/brands/generate.mjs` pins four points in its history (releases 11.15.0, 15.22.0 and 16.3.0, plus commit 90646e4), because Simple Icons removes marks over time (Microsoft, AWS and Slack are only in the oldest, newer ones only in the later releases). Each brand takes the first pinned version that has it.
- The brand list is `scripts/brands/brands.mjs` and `brands.extra.mjs` (id, name, Simple Icons slug, aliases, domains, category). The generator writes `src/brands/registry.generated.ts` (names, aliases, domains) and one lazy `paths.<category>.generated.ts` chunk per category. Run `node scripts/brands/generate.mjs` from `apps/web` to refresh.
- Marks that Simple Icons does not carry, or has removed, are never redrawn or approximated. The brand stays in the registry (so names, aliases and sender domains still resolve) and draws a neutral letter tile with a hue derived from its name. No documented brand colour is recorded for these, so the hue is not brand-specific.

## Supplying an official logo

For a brand shown as a monogram below, drop an official SVG at `apps/web/src/brands/custom/<id>.svg` and add an entry to `apps/web/src/brands/custom/manifest.json`:

```json
{ "cgd": { "source": "https://... press kit or brand page the file came from", "licence": "permission granted by the owner / CC0 / ..." } }
```

`generate.mjs` refuses a file without both fields, or whose id is not a registry id. The file is sanitised, never stripped: only `svg`, `g`, `path`, `circle`, `rect` and `polygon` with plain geometry attributes are accepted, with `viewBox="0 0 24 24"`. Script, style, `href`, `use`, `image`, `foreignObject`, event handlers, transforms, fills, entities, comments and text are errors. Shapes are merged into one path. A test (`registry.test.tsx`) re-checks every supplied file, and the manifest.

## Trademarks

All names and logos are trademarks or registered trademarks of their owners. They are shown only to identify the service a setting refers to. Their presence does not imply endorsement by, or affiliation with, those companies, and the CC0 licence of the path data does not grant any trademark rights.

## Privacy and security

Logos are bundled with the app as inline SVG path data. Nothing is fetched from a CDN, favicon service or the provider itself, so showing a logo never reveals which providers an operator uses. The path data is loaded lazily, one chunk per category, the first time a logo of that category is drawn. A test checks that no entry contains a URL, script or event handler.

## Using it

```tsx
import {BrandLogo, BrandLabel, BrandCell, HostBrandCell} from '../brands/BrandLogo.tsx';
<BrandLogo brand="gmail.com" size={24} />       // standalone: role="img", labelled
<BrandLabel brand="okta" />                      // logo + visible name, logo is decorative
<BrandCell brand="github">...content...</BrandCell>
<HostBrandCell host={url}>...</HostBrandCell>   // only when the host is a known brand
```

`brand` resolves by id, display name, alias or domain. `brandForHost()` matches host names, URLs and e-mail addresses by domain suffix, and `brandInText()` finds a brand inside a free-text label. Sizes are 16, 20, 24 and 32. Single-colour marks follow the text colour unless the brand colour reads on both light and dark surfaces; `mono` forces the text colour. In forced-colors mode every logo uses system colours.

## Domain matching

`brandForHost()` matches a host, URL or e-mail address by whole registrable-domain suffix: `mail.paypal.com` matches `paypal.com`; `paypal.com.evil.io`, `evilpaypal.com`, `paypal.com-secure.net` and `https://evil.io/?u=@paypal.com` do not. Only the host part is read (scheme, path, query and `user@` are removed first), and the longest matching suffix wins, so `aws.amazon.com` is AWS while `amazon.co.uk` is Amazon. Tests cover the lookalike cases. Subdomains of shared-hosting domains (`run.app`, `appspot.com`, `azurewebsites.net`, `windows.net`) are deliberately not registered.

## Brand logos beside verified senders

In the reader, the logo of a recognised brand is shown beside the sender only when the message passed an aligned DMARC check and DKIM passed for a domain of the same brand (`brandAvatarFor`, `SenderBrandMark`). Brands without a bundled mark show nothing there. The setting `mail.brandAvatars` (user, domain and global scope, default on) turns it off. The message list carries no check result, so it keeps the normal avatar. The separate older `contacts.senderIcons` setting still shows bundled sender-domain icons in lists as appearance hints; it is documented as not being authentication evidence.

## Interpretation of ambiguous names

Names in the product lists that were typos or ambiguous were resolved as follows. The misspelling is kept as an alias so it still resolves.

| Written | Read as | Note |
| --- | --- | --- |
| abibaba | Alibaba | `alibaba.com`, `alibabacloud.com`, `aliyun.com` |
| gogcom | GOG.com | |
| gleam | Gleam (programming language, `gleam.run`) | The dataset only has the language. `gleam.io` (the giveaway service) is a separate entry with a monogram, so its mails never show the language's logo |
| paxb | Pax8 | |
| vvalley | V-Valley | |
| arrows ecs | Arrow ECS | `arrowecs.com` only, not `arrow.com` (Arrow Electronics) |
| teasmpeak | TeamSpeak | |
| theter ush | Tether | |
| blockahincom | Blockchain.com | |
| weromexico | Aeroméxico | |
| also | ALSO Group | `also.com` |
| commerbank | Commerzbank | |
| opencode | OpenCode (`opencode.ai`) | |
| ring | Ring (`ring.com`) | |
| z.ai | Z.ai (`z.ai`) | |
| cuf | CUF (`cuf.pt`) | |
| dinesplus, seezer | Disney+, Deezer | |
| Grok (xAI) | Grok, `x.ai` and `grok.com` | |
| Dell EMC | separate from Dell | `dellemc.com`, `emc.com` |
| PlayStation Network (PSN) | separate from Sony | `playstation.com`, `sonyentertainmentnetwork.com` |
| Square/Block | Block | `block.xyz`, `squareup.com` |
| Merck, MSD | two entries | Merck KGaA (`merckgroup.com`, `emdgroup.com`) and MSD (`msd.com`, `merck.com`) |
| mo, mta(uk) | not added | Not unambiguous |
| vig consulting | not added | "big consulting" is not a single identifiable firm |

Not added because a sender domain could not be confirmed (no guesses): Banco Português de Gestão, SAMS, Grupo Trofa Saúde, HPA Saúde, CLIP, Gesaúde, 7Seasons, AMA, Clínica Sagres. Several Portuguese companies share a group domain with a parent and resolve through it.

Insurance and eyewear batch:

| Written | Read as | Note |
| --- | --- | --- |
| logo segurso | Logo Seguro (`logo.pt`) | "Logo" alone also resolves |
| OK Seguros, ok-seguros | OK! Teleseguros (`okteleseguros.pt`) | |
| Anthem / Elevance | one entry, Elevance Health | `elevancehealth.com`, `anthem.com` |
| Prudential | two entries | Prudential Financial (US, `prudential.com`) and Prudential plc (UK, `prudentialplc.com`) |
| Zurich | two entries | Zurich Insurance (`zurich.com`, `zurich.co.uk`) and Zurich Portugal (`zurich.pt`) |
| Allianz | three entries | Allianz, Allianz Care, Allianz Portugal |
| Nationwide | US insurer only (`nationwide.com`) | The UK building society is not included |
| Tom Ford eyewear | Tom Ford (`tomford.com`) | The brand domain, not an eyewear-only one |
| Pearle, Pearle Vision | Pearle Vision | |
| Persol, Oakley | listed twice in the request | one entry each |

Not added in this batch, because a sender domain could not be confirmed (no guesses): Sabadell Seguros, Ocidental, Real Vida, Police, Lensway, Matsuda, Face a Face, Oticas Plus, and the "etc." in the insurance list. The Simple Icons dataset has none of the insurers, Moey! or eyewear brands, so all of them use the tile.

## Size cap

Path data for one brand is limited to 8,000 characters. Very detailed crests (Porsche, Ferrari, Ritz-Carlton, Peugeot, Wipro, MSI) exceed it and show the monogram tile, which keeps the total of all category chunks near 195 KB gzip.

## Provider labels

Names typed by an operator (sign-in provider names, AI profile labels) only resolve to brands in provider-type categories (`PROVIDER_CATEGORIES`), so a provider called "Delta" or "Target" does not get an airline or retailer logo. Hosts and ids resolve in every category.

## Coverage

380 real marks, 432 monograms, 812 brands. Brands shown as a monogram are the candidates for a supplied official SVG:

Yahoo, Fastmail, Yandex, Linode, UnifiedPush, reCAPTCHA, hCaptcha, MSN, Globo, Amazon Business, Temu, Shein, OLX, Worten, Pingo Doce, Decathlon, Microsoft 365, Google Workspace, Monday.com, Gameforge, Kakao Games, Bandai Namco, ZeniMax, LEGO, Disney+, Santander, Caixa Geral de Depósitos, MB WAY, ActivoBank, Banco BPI, Banco CTT, Novo Banco, Trade Republic, Kraken, Skrill, Monese, Chime, eToro, Millennium bcp, Montepio, Crédito Agrícola, Bankinter, ING, BNP Paribas, Lloyds Bank, Citi, CTT, GLS, TNT, Royal Mail, InPost, MRW, SEUR, Chronopost, Correos, Evri, Geopost, Maersk, TAP Air Portugal, FlixBus, Trainline, CP Comboios de Portugal, BlaBlaCar, Via Verde, Brisa, Galp, Endesa, Águas de Portugal, EPAL, ePortugal, IRN, República Portuguesa, DECO PROTESTE, HPE, Aruba, Dell EMC, MSI, Arista Networks, SolarWinds, Equinix, Hurricane Electric, Grok, Gleam.io, Databox, Kalshi, Polymarket, TD SYNNEX, Ingram Micro, ALSO Group, Esprinet, V-Valley, Arrow ECS, Westcon-Comstor, Pax8, Sherweb, Infortisa, PwC, Mazars, Deloitte, EY, KPMG, McKinsey & Company, BCG, Bain & Company, Capgemini, Wipro, Booz Allen, Roland Berger, Oliver Wyman, Mercer, Aon, Willis Towers Watson, MSD, Pfizer, Novartis, Roche, Sanofi, AstraZeneca, Bayer, Novo Nordisk, The New York Times, BBC, Reuters, Bloomberg, Financial Times, Wall Street Journal, Le Monde, El País, Corriere della Sera, Público, Observador, Expresso, Diário de Notícias, Jornal de Notícias, Correio da Manhã, SIC, TVI, RTP, SAPO, Notícias ao Minuto, Eurosport, Euronews, The Economist, Le Figaro, El Mundo, la Repubblica, Die Zeit, Bild, Jornal de Negócios, Record, A Bola, Sonae, Primark, C&A, Mango, Pull&Bear, Bershka, Lacoste, Levi's, Gucci, Louis Vuitton, Chanel, Prada, Burberry, Versace, Balenciaga, Cartier, Rolex, Ralph Lauren, Tommy Hilfiger, Calvin Klein, Hugo Boss, Armani, Valentino, Fendi, Domino's, Pizza Hut, Subway, Bolt, Bolt Food, Porsche, Ferrari, Peugeot, Lexus, BYD, Rivian, Bumble, Hinge, Grindr, Match, Feeld, eHarmony, Plenty of Fish, Happn, Meetic, Aeroméxico, Vueling, SWISS, Ritz-Carlton, Hyatt, Accor, IHG, Julius Baer, UBS, Credit Suisse, Pictet, Lombard Odier, J.P. Morgan, Morgan Stanley, Coutts, Banco Santander Totta, Banco Carregosa, Banco Invest, Banco BiG, Worldpay, Checkout.com, Authorize.net, Mollie, PayU, Nuvei, Rapyd, Ifthenpay, EuPago, SIBS, Easypay, Pagar.me, UnionPay, CUF, Luz Saúde, Lusíadas Saúde, Medicare, Synlab, Unilabs, Joaquim Chaves Saúde, Affidea, Mota-Engil, Teixeira Duarte, Casais, Somague, Alves Ribeiro, Ferrovial, Sacyr, Acciona, Vinci, Bouygues, Skanska, Grupo Soares da Costa, Cimpor, Secil, Navigator, EDP, REN, Altri, Corticeira Amorim, Jerónimo Martins, NOS, MEO, NOWO, Altice, Comcast, Iberdrola, Repsol, BP, TotalEnergies, Engie, Enel, Costco, Best Buy, The Home Depot, Mercado Libre, idealista, Imovirtual, KuantoKusta, Autoridade Tributária e Aduaneira, Segurança Social, Serviço Nacional de Saúde, Rabobank, ABN AMRO, BBVA, UniCredit, Intesa Sanpaolo, Itaú, Bradesco, NatWest, Moey!, Fidelidade, Ageas, MetLife, Logo Seguro, AdvanceCare, Multicare, OK! Teleseguros, Liberty Seguros, Tranquilidade, UNA Seguros, Médis, Zurich Portugal, Allianz Portugal, Lusitania Seguros, Victoria Seguros, Allianz, Allianz Care, AXA, Generali, Zurich Insurance, Aviva, Prudential Financial, Prudential plc, Chubb, AIG, Munich Re, Swiss Re, Hannover Re, SCOR, Mapfre, Cigna, Aetna, UnitedHealthcare, Elevance Health, Humana, Kaiser Permanente, Blue Cross Blue Shield, Bupa, Vitality, Direct Line, Admiral, LV=, Legal & General, Aegon, NN Group, Achmea, Talanx, ERGO, HDI, Covéa, Groupama, MAIF, MACIF, Crédit Agricole Assurances, CNP Assurances, Sampo, If P&C Insurance, Tryg, Folksam, Gjensidige, Mutua Madrileña, Caser, Berkshire Hathaway, State Farm, GEICO, Progressive, Allstate, Liberty Mutual, Nationwide, USAA, Travelers, Farmers Insurance, Lemonade, Porto Seguro, Bradesco Seguros, SulAmérica, ICBC, Ping An, China Life, Tokio Marine, Sompo, MS&AD, Dai-ichi Life, Manulife, Sun Life, Great-West Lifeco, Linda Farrow, Ray-Ban, Oakley, Luxottica, EssilorLuxottica, Essilor, Safilo, Marcolin, De Rigo, Kering Eyewear, Maui Jim, Persol, Warby Parker, Silhouette, Mykita, Oliver Peoples, Carrera, Costa Del Mar, Smith Optics, Tom Ford, Cutler and Gross, Barton Perreira, Garrett Leight, Moscot, Lindberg, ic! berlin, ZEISS, Hoya, Rodenstock, Varilux, Transitions, GrandVision, Vision Express, Specsavers, Multiopticas, LensCrafters, Pearle Vision, Fielmann, Apollo-Optik, Zenni Optical, GlassesUSA, EyeBuyDirect, Mister Spex, Afflelou, Optical Center, Alain Mikli, Gentle Monster, Vuarnet, Bollé, Julbo, Spy Optic, VonZipper, TAG Heuer, DITA, Etnia Barcelona, Ørgreen, Kuboraum, EYEVAN, Jacques Marie Mage, Cazal, Lafont.

### ai

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| OpenAI | `openai` | real mark | openai.com |
| Anthropic | `anthropic` | real mark | anthropic.com |
| Google Gemini | `gemini` | real mark | generativelanguage.googleapis.com, gemini.google.com |
| Mistral AI | `mistral` | real mark | mistral.ai |
| Ollama | `ollama` | real mark | ollama.com, ollama.ai |
| Perplexity | `perplexity` | real mark | perplexity.ai |
| Hugging Face | `huggingface` | real mark | huggingface.co |
| OpenCode | `opencode` | real mark | opencode.ai |
| Grok | `grok` | monogram | x.ai, grok.com |
| Z.ai | `zai` | real mark | z.ai, zhipuai.cn |
| ElevenLabs | `elevenlabs` | real mark | elevenlabs.io |

### analytics

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Plausible | `plausible` | real mark | plausible.io |
| Matomo | `matomo` | real mark | matomo.cloud, matomo.org |
| Google Analytics | `googleanalytics` | real mark | google-analytics.com, analytics.google.com, googletagmanager.com |
| PostHog | `posthog` | real mark | posthog.com |
| Umami | `umami` | real mark | umami.is |
| Fathom | `fathom` | real mark | usefathom.com |

### app

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Slack | `slack` | real mark | slack.com |
| Microsoft Teams | `teams` | real mark | teams.microsoft.com, webhook.office.com |
| Discord | `discord` | real mark | discord.com, discordapp.com |
| Zapier | `zapier` | real mark | zapier.com |
| Matrix | `matrix` | real mark | matrix.org |
| Mattermost | `mattermost` | real mark | mattermost.com |
| Telegram | `telegram` | real mark | telegram.org, t.me |
| Jira | `jira` | real mark | atlassian.net, jira.com |
| Notion | `notion` | real mark | notion.so, notion.com |
| Trello | `trello` | real mark | trello.com |

### auto

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Tesla | `tesla` | real mark | tesla.com |
| Mercedes-Benz | `mercedes` | real mark | mercedes-benz.com, mercedes-benz.pt |
| BMW | `bmw` | real mark | bmw.com |
| Audi | `audi` | real mark | audi.com |
| Volkswagen | `volkswagen` | real mark | volkswagen.com, vw.com |
| Porsche | `porsche` | monogram | porsche.com |
| Ferrari | `ferrari` | monogram | ferrari.com |
| Fiat | `fiat` | real mark | fiat.com |
| Toyota | `toyota` | real mark | toyota.com, toyota.eu |
| Honda | `honda` | real mark | honda.com |
| Nissan | `nissan` | real mark | nissan.com, nissan-global.com |
| Ford | `ford` | real mark | ford.com |
| Renault | `renault` | real mark | renault.com |
| Peugeot | `peugeot` | monogram | peugeot.com |
| Citroën | `citroen` | real mark | citroen.com |
| Volvo | `volvo` | real mark | volvocars.com |
| Kia | `kia` | real mark | kia.com |
| Hyundai | `hyundai` | real mark | hyundai.com |
| SEAT | `seat` | real mark | seat.com |
| Škoda | `skoda` | real mark | skoda-auto.com |
| Mini | `mini` | real mark | mini.com |
| Opel | `opel` | real mark | opel.com |
| Mazda | `mazda` | real mark | mazda.com |
| Subaru | `subaru` | real mark | subaru.com |
| Lexus | `lexus` | monogram | lexus.com |
| Jaguar | `jaguar` | real mark | jaguar.com |
| Land Rover | `landrover` | real mark | landrover.com |
| Alfa Romeo | `alfaromeo` | real mark | alfaromeo.com |
| Lamborghini | `lamborghini` | real mark | lamborghini.com |
| BYD | `byd` | monogram | byd.com |
| Rivian | `rivian` | monogram | rivian.com |
| John Deere | `johndeere` | real mark | deere.com |
| AutoZone | `autozone` | real mark | autozone.com |

### channel

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| TD SYNNEX | `tdsynnex` | monogram | tdsynnex.com, synnex.com |
| Ingram Micro | `ingrammicro` | monogram | ingrammicro.com |
| ALSO Group | `also` | monogram | also.com |
| Esprinet | `esprinet` | monogram | esprinet.com |
| V-Valley | `vvalley` | monogram | v-valley.com |
| Arrow ECS | `arrowecs` | monogram | arrowecs.com |
| Westcon-Comstor | `comstor` | monogram | westcon.com, comstor.com |
| Pax8 | `pax8` | monogram | pax8.com |
| Sherweb | `sherweb` | monogram | sherweb.com |
| Infortisa | `infortisa` | monogram | infortisa.com |

### cloud

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Amazon Web Services | `aws` | real mark | amazonaws.com, aws.amazon.com, amazonses.com, awsapps.com |
| Google Cloud | `googlecloud` | real mark | cloud.google.com, storage.googleapis.com |
| Microsoft Azure | `azure` | real mark | azure.com, openai.azure.com, microsoftazure.com |
| Cloudflare | `cloudflare` | real mark | cloudflare.com, cloudflare-dns.com, cloudflarestorage.com, cloudflareinsights.com |
| DigitalOcean | `digitalocean` | real mark | digitalocean.com, digitaloceanspaces.com |
| Hetzner | `hetzner` | real mark | hetzner.com, hetzner.cloud, your-server.de, your-storagebox.de |
| OVHcloud | `ovh` | real mark | ovh.com, ovh.net, ovhcloud.com |
| Vultr | `vultr` | real mark | vultr.com |
| Linode | `linode` | monogram | linode.com, linodeobjects.com |
| Akamai | `akamai` | real mark | akamai.com, akamaiedge.net |
| Fastly | `fastly` | real mark | fastly.com, fastly.net |
| Hostinger | `hostinger` | real mark | hostinger.com, hostinger.pt, hostinger.es |
| GoDaddy | `godaddy` | real mark | godaddy.com, secureserver.net |
| Vercel | `vercel` | real mark | vercel.com |
| Netlify | `netlify` | real mark | netlify.com |
| JetBrains | `jetbrains` | real mark | jetbrains.com |
| Docker | `docker` | real mark | docker.com |
| npm | `npm` | real mark | npmjs.com, npmjs.org |
| Platform.sh | `platformsh` | real mark | platform.sh |
| Codeberg | `codeberg` | real mark | codeberg.org |
| Gitea | `gitea` | real mark | gitea.com, gitea.io |
| PyPI | `pypi` | real mark | pypi.org, python.org |
| Let's Encrypt | `letsencrypt` | real mark | letsencrypt.org |
| Spaceship | `spaceship` | real mark | spaceship.com |
| Namecheap | `namecheap` | real mark | namecheap.com |
| Heroku | `heroku` | real mark | heroku.com |

### dating

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Tinder | `tinder` | real mark | tinder.com, gotinder.com |
| Bumble | `bumble` | monogram | bumble.com |
| Badoo | `badoo` | real mark | badoo.com |
| Hinge | `hinge` | monogram | hinge.co |
| Grindr | `grindr` | monogram | grindr.com |
| OkCupid | `okcupid` | real mark | okcupid.com |
| Match | `match` | monogram | match.com, mtch.com |
| Feeld | `feeld` | monogram | feeld.co |
| eHarmony | `eharmony` | monogram | eharmony.com |
| Plenty of Fish | `pof` | monogram | pof.com, plentyoffish.com |
| Happn | `happn` | monogram | happn.com, happn.fr |
| Meetic | `meetic` | monogram | meetic.com, meetic.fr |

### eyewear

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Linda Farrow | `lindafarrow` | monogram | lindafarrow.com |
| Ray-Ban | `rayban` | monogram | ray-ban.com |
| Oakley | `oakley` | monogram | oakley.com |
| Luxottica | `luxottica` | monogram | luxottica.com |
| EssilorLuxottica | `essilorluxottica` | monogram | essilorluxottica.com |
| Essilor | `essilor` | monogram | essilor.com |
| Safilo | `safilo` | monogram | safilo.com |
| Marcolin | `marcolin` | monogram | marcolin.com |
| De Rigo | `derigo` | monogram | derigo.com |
| Kering Eyewear | `keringeyewear` | monogram | keringeyewear.com |
| Maui Jim | `mauijim` | monogram | mauijim.com |
| Persol | `persol` | monogram | persol.com |
| Warby Parker | `warbyparker` | monogram | warbyparker.com |
| Silhouette | `silhouette` | monogram | silhouette.com |
| Mykita | `mykita` | monogram | mykita.com |
| Oliver Peoples | `oliverpeoples` | monogram | oliverpeoples.com |
| Carrera | `carrera` | monogram | carreraworld.com |
| Costa Del Mar | `costadelmar` | monogram | costadelmar.com |
| Smith Optics | `smithoptics` | monogram | smithoptics.com |
| Tom Ford | `tomford` | monogram | tomford.com, tomfordfashion.com |
| Cutler and Gross | `cutlerandgross` | monogram | cutlerandgross.com |
| Barton Perreira | `bartonperreira` | monogram | bartonperreira.com |
| Garrett Leight | `garrettleight` | monogram | garrettleight.com |
| Moscot | `moscot` | monogram | moscot.com |
| Lindberg | `lindberg` | monogram | lindberg.com |
| ic! berlin | `icberlin` | monogram | ic-berlin.de, ic-berlin.com |
| ZEISS | `zeiss` | monogram | zeiss.com |
| Hoya | `hoya` | monogram | hoya.com |
| Rodenstock | `rodenstock` | monogram | rodenstock.com |
| Varilux | `varilux` | monogram | varilux.com |
| Transitions | `transitions` | monogram | transitions.com |
| GrandVision | `grandvision` | monogram | grandvision.com |
| Vision Express | `visionexpress` | monogram | visionexpress.com |
| Specsavers | `specsavers` | monogram | specsavers.com, specsavers.co.uk |
| Multiopticas | `multiopticas` | monogram | multiopticas.pt |
| LensCrafters | `lenscrafters` | monogram | lenscrafters.com |
| Pearle Vision | `pearlevision` | monogram | pearlevision.com |
| Fielmann | `fielmann` | monogram | fielmann.com, fielmann.de |
| Apollo-Optik | `apollooptik` | monogram | apollo.de |
| Zenni Optical | `zenni` | monogram | zennioptical.com |
| GlassesUSA | `glassesusa` | monogram | glassesusa.com |
| EyeBuyDirect | `eyebuydirect` | monogram | eyebuydirect.com |
| Mister Spex | `misterspex` | monogram | misterspex.de, misterspex.com |
| Afflelou | `afflelou` | monogram | afflelou.com, alainafflelou.com |
| Optical Center | `opticalcenter` | monogram | optical-center.fr |
| Alain Mikli | `alainmikli` | monogram | mikli.com |
| Gentle Monster | `gentlemonster` | monogram | gentlemonster.com |
| Vuarnet | `vuarnet` | monogram | vuarnet.com |
| Bollé | `bolle` | monogram | bolle.com, bolle-brands.com |
| Julbo | `julbo` | monogram | julbo.com |
| Spy Optic | `spyoptic` | monogram | spyoptic.com |
| VonZipper | `vonzipper` | monogram | vonzipper.com |
| TAG Heuer | `tagheuer` | monogram | tagheuer.com |
| DITA | `dita` | monogram | dita.com |
| Etnia Barcelona | `etniabarcelona` | monogram | etniabarcelona.com |
| Ørgreen | `orgreen` | monogram | orgreen.com |
| Kuboraum | `kuboraum` | monogram | kuboraum.com |
| EYEVAN | `eyevan` | monogram | eyevan.com, eyevan7285.com |
| Jacques Marie Mage | `jacquesmariemage` | monogram | jacquesmariemage.com |
| Cazal | `cazal` | monogram | cazal.com |
| Lafont | `lafont` | monogram | lafont.com |

### fashion

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Nike | `nike` | real mark | nike.com |
| Adidas | `adidas` | real mark | adidas.com |
| Puma | `puma` | real mark | puma.com |
| Uniqlo | `uniqlo` | real mark | uniqlo.com |
| H&M | `hm` | real mark | hm.com |
| Zara | `zara` | real mark | zara.com, inditex.com |
| Primark | `primark` | monogram | primark.com |
| C&A | `ca` | monogram | c-and-a.com |
| Mango | `mango` | monogram | mango.com |
| Pull&Bear | `pullandbear` | monogram | pullandbear.com |
| Bershka | `bershka` | monogram | bershka.com |
| Lacoste | `lacoste` | monogram | lacoste.com |
| Levi's | `levis` | monogram | levi.com, levistrauss.com |
| Under Armour | `underarmour` | real mark | underarmour.com |
| New Balance | `newbalance` | real mark | newbalance.com |
| Reebok | `reebok` | real mark | reebok.com |
| The North Face | `thenorthface` | real mark | thenorthface.com |
| Gucci | `gucci` | monogram | gucci.com |
| Louis Vuitton | `louisvuitton` | monogram | louisvuitton.com |
| Chanel | `chanel` | monogram | chanel.com |
| Prada | `prada` | monogram | prada.com |
| Hermès | `hermes` | real mark | hermes.com |
| Dior | `dior` | real mark | dior.com |
| Burberry | `burberry` | monogram | burberry.com |
| Versace | `versace` | monogram | versace.com |
| Balenciaga | `balenciaga` | monogram | balenciaga.com |
| Cartier | `cartier` | monogram | cartier.com |
| Rolex | `rolex` | monogram | rolex.com |
| Ralph Lauren | `ralphlauren` | monogram | ralphlauren.com |
| Tommy Hilfiger | `tommyhilfiger` | monogram | tommy.com, tommyhilfiger.com |
| Calvin Klein | `calvinklein` | monogram | calvinklein.com |
| Hugo Boss | `hugoboss` | monogram | hugoboss.com |
| Armani | `armani` | monogram | armani.com |
| Valentino | `valentino` | monogram | valentino.com |
| Fendi | `fendi` | monogram | fendi.com |

### finance

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| PayPal | `paypal` | real mark | paypal.com, paypal.pt, paypal.es, paypal.de, ... |
| Wise | `wise` | real mark | wise.com, transferwise.com |
| Revolut | `revolut` | real mark | revolut.com |
| Visa | `visa` | real mark | visa.com, visaeurope.com |
| Mastercard | `mastercard` | real mark | mastercard.com |
| Stripe | `stripe` | real mark | stripe.com |
| American Express | `amex` | real mark | americanexpress.com, aexp.com |
| Monzo | `monzo` | real mark | monzo.com |
| Santander | `santander` | monogram | santander.pt, santander.com, santander.es, santander.co.uk |
| Caixa Geral de Depósitos | `cgd` | monogram | cgd.pt, caixa.pt |
| MB WAY | `mbway` | monogram | mbway.pt, mbway.com |
| ActivoBank | `activobank` | monogram | activobank.pt, activobanco.pt |
| Banco BPI | `bpi` | monogram | bpi.pt, bancobpi.pt |
| Banco CTT | `bancoctt` | monogram | bancoctt.pt |
| Novo Banco | `novobanco` | monogram | novobanco.pt |
| Western Union | `westernunion` | real mark | westernunion.com |
| Trade Republic | `traderepublic` | monogram | traderepublic.com |
| Klarna | `klarna` | real mark | klarna.com |
| N26 | `n26` | real mark | n26.com |
| Nubank | `nubank` | real mark | nubank.com.br, nu.com.mx |
| Cash App | `cashapp` | real mark | cash.app |
| Venmo | `venmo` | real mark | venmo.com |
| Coinbase | `coinbase` | real mark | coinbase.com |
| Binance | `binance` | real mark | binance.com |
| Kraken | `kraken` | monogram | kraken.com |
| Payoneer | `payoneer` | real mark | payoneer.com |
| Skrill | `skrill` | monogram | skrill.com |
| Monese | `monese` | monogram | monese.com |
| bunq | `bunq` | real mark | bunq.com |
| Starling Bank | `starling` | real mark | starlingbank.com |
| Chime | `chime` | monogram | chime.com |
| Robinhood | `robinhood` | real mark | robinhood.com |
| eToro | `etoro` | monogram | etoro.com |
| Millennium bcp | `millenniumbcp` | monogram | millenniumbcp.pt, bcp.pt, ptmillennium.pt |
| Montepio | `montepio` | monogram | montepio.pt, bancomontepio.pt |
| Crédito Agrícola | `creditoagricola` | monogram | creditoagricola.pt |
| Bankinter | `bankinter` | monogram | bankinter.pt, bankinter.com, bankinter.es |
| ING | `ing` | monogram | ing.com, ing.nl, ing.de, ing.be, ... |
| BNP Paribas | `bnpparibas` | monogram | bnpparibas.com, bnpparibas.pt, bnpparibas.fr |
| Deutsche Bank | `deutschebank` | real mark | deutsche-bank.de, db.com, deutschebank.com |
| HSBC | `hsbc` | real mark | hsbc.com, hsbc.co.uk, hsbc.fr |
| Barclays | `barclays` | real mark | barclays.com, barclays.co.uk |
| Lloyds Bank | `lloyds` | monogram | lloydsbank.com, lloydsbank.co.uk, lloydsbankinggroup.com |
| Commerzbank | `commerzbank` | real mark | commerzbank.de, commerzbank.com |
| Sparkasse | `sparkasse` | real mark | sparkasse.de, s-sparkasse.de |
| Wells Fargo | `wellsfargo` | real mark | wellsfargo.com |
| Chase | `chase` | real mark | chase.com |
| Citi | `citi` | monogram | citi.com, citibank.com |
| Bank of America | `bankofamerica` | real mark | bankofamerica.com, bofa.com |
| Trezor | `trezor` | real mark | trezor.io |
| Tether | `tether` | real mark | tether.to, tether.io |
| Blockchain.com | `blockchaincom` | real mark | blockchain.com |
| Kalshi | `kalshi` | monogram | kalshi.com |
| Polymarket | `polymarket` | monogram | polymarket.com |
| WebMoney | `webmoney` | real mark | webmoney.ru, wmtransfer.com |
| Julius Baer | `juliusbaer` | monogram | juliusbaer.com |
| UBS | `ubs` | monogram | ubs.com |
| Credit Suisse | `creditsuisse` | monogram | credit-suisse.com |
| Pictet | `pictet` | monogram | pictet.com |
| Lombard Odier | `lombardodier` | monogram | lombardodier.com |
| J.P. Morgan | `jpmorgan` | monogram | jpmorgan.com, jpmorganchase.com |
| Goldman Sachs | `goldmansachs` | real mark | goldmansachs.com, gs.com |
| Morgan Stanley | `morganstanley` | monogram | morganstanley.com |
| Coutts | `coutts` | monogram | coutts.com |
| Banco Santander Totta | `santandertotta` | monogram | santandertotta.pt |
| Banco Carregosa | `bancocarregosa` | monogram | bancocarregosa.com, bancocarregosa.pt |
| Banco Invest | `bancoinvest` | monogram | bancoinvest.pt |
| Banco BiG | `bancobig` | monogram | bancobig.pt |
| Rabobank | `rabobank` | monogram | rabobank.com, rabobank.nl |
| ABN AMRO | `abnamro` | monogram | abnamro.nl, abnamro.com |
| BBVA | `bbva` | monogram | bbva.com, bbva.es |
| CaixaBank | `caixabank` | real mark | caixabank.com, caixabank.es |
| UniCredit | `unicredit` | monogram | unicredit.it, unicreditgroup.eu |
| Intesa Sanpaolo | `intesasanpaolo` | monogram | intesasanpaolo.com |
| Itaú | `itau` | monogram | itau.com.br |
| Bradesco | `bradesco` | monogram | bradesco.com.br |
| NatWest | `natwest` | monogram | natwest.com |
| Moey! | `moey` | monogram | moey.pt |

### food

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Taco Bell | `tacobell` | real mark | tacobell.com |
| McDonald's | `mcdonalds` | real mark | mcdonalds.com |
| Burger King | `burgerking` | real mark | burgerking.com, bk.com |
| KFC | `kfc` | real mark | kfc.com |
| Starbucks | `starbucks` | real mark | starbucks.com |
| Domino's | `dominos` | monogram | dominos.com |
| Pizza Hut | `pizzahut` | monogram | pizzahut.com |
| Subway | `subway` | monogram | subway.com |
| Deliveroo | `deliveroo` | real mark | deliveroo.com, deliveroo.co.uk |
| Just Eat | `justeat` | real mark | just-eat.com, justeat.com, just-eat.co.uk |

### gaming

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Roblox | `roblox` | real mark | roblox.com |
| Gameforge | `gameforge` | monogram | gameforge.com |
| Steam | `steam` | real mark | steampowered.com, steamcommunity.com, valvesoftware.com |
| Tencent | `tencent` | real mark | tencent.com, qq.com |
| Square Enix | `squareenix` | real mark | square-enix.com, square-enix-games.com |
| Activision Blizzard | `activisionblizzard` | real mark | activision.com, blizzard.com, battle.net |
| Kakao Games | `kakaogames` | monogram | kakaogames.com |
| Bandai Namco | `bandainamco` | monogram | bandainamcoent.com, bandainamco.com |
| Sony | `sony` | real mark | sony.com |
| Riot Games | `riotgames` | real mark | riotgames.com |
| ZeniMax | `zenimax` | monogram | zenimax.com, bethesda.net |
| Epic Games | `epicgames` | real mark | epicgames.com |
| Nintendo | `nintendo` | real mark | nintendo.com, nintendo.net |
| LEGO | `lego` | monogram | lego.com |
| TeamSpeak | `teamspeak` | real mark | teamspeak.com |
| Supercell | `supercell` | real mark | supercell.com |
| GOG.com | `gog` | real mark | gog.com |
| PlayStation Network | `playstation` | real mark | playstation.com, sonyentertainmentnetwork.com |

### health

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| CUF | `cuf` | monogram | cuf.pt |
| Luz Saúde | `luzsaude` | monogram | luzsaude.pt |
| Lusíadas Saúde | `lusiadas` | monogram | lusiadas.pt |
| Medicare | `medicare` | monogram | medicare.pt |
| Synlab | `synlab` | monogram | synlab.pt, synlab.com |
| Unilabs | `unilabs` | monogram | unilabs.pt, unilabs.com |
| Joaquim Chaves Saúde | `jcs` | monogram | jcs.pt |
| Affidea | `affidea` | monogram | affidea.pt, affidea.com |

### identity

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Google | `google` | real mark | google.com, googleapis.com, accounts.google.com |
| Microsoft | `microsoft` | real mark | microsoft.com, microsoftonline.com |
| Apple | `apple` | real mark | apple.com, push.apple.com |
| Okta | `okta` | real mark | okta.com, oktapreview.com |
| Auth0 | `auth0` | real mark | auth0.com |
| Keycloak | `keycloak` | real mark |  |
| authentik | `authentik` | real mark |  |
| GitHub | `github` | real mark | github.com, githubusercontent.com |
| GitLab | `gitlab` | real mark | gitlab.com |

### industry

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Mota-Engil | `motaengil` | monogram | mota-engil.pt, mota-engil.com |
| Teixeira Duarte | `teixeiraduarte` | monogram | teixeiraduarte.pt, teixeiraduarte.com |
| Casais | `casais` | monogram | casais.pt |
| Somague | `somague` | monogram | somague.pt |
| Alves Ribeiro | `alvesribeiro` | monogram | alvesribeiro.pt |
| Ferrovial | `ferrovial` | monogram | ferrovial.com |
| Sacyr | `sacyr` | monogram | sacyr.com |
| Acciona | `acciona` | monogram | acciona.com |
| Vinci | `vinci` | monogram | vinci.com |
| Bouygues | `bouygues` | monogram | bouygues.com |
| Skanska | `skanska` | monogram | skanska.com |
| Grupo Soares da Costa | `soaresdacosta` | monogram | soaresdacosta.pt |
| Cimpor | `cimpor` | monogram | cimpor.com |
| Secil | `secil` | monogram | secil.pt |
| Navigator | `navigator` | monogram | thenavigatorcompany.com |
| Altri | `altri` | monogram | altri.pt |
| Corticeira Amorim | `amorim` | monogram | amorim.com, corticeiraamorim.com |

### insurance

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Fidelidade | `fidelidade` | monogram | fidelidade.pt |
| Ageas | `ageas` | monogram | ageas.pt, ageas.com |
| MetLife | `metlife` | monogram | metlife.com, metlife.pt |
| Logo Seguro | `logoseguro` | monogram | logo.pt |
| AdvanceCare | `advancecare` | monogram | advancecare.pt |
| Multicare | `multicare` | monogram | multicare.pt |
| OK! Teleseguros | `okseguros` | monogram | okteleseguros.pt |
| Liberty Seguros | `liberty` | monogram | libertyseguros.pt, libertyseguros.es |
| Tranquilidade | `tranquilidade` | monogram | tranquilidade.pt |
| UNA Seguros | `unaseguros` | monogram | unaseguros.pt |
| Médis | `medis` | monogram | medis.pt |
| Zurich Portugal | `zurichportugal` | monogram | zurich.pt |
| Allianz Portugal | `allianzportugal` | monogram | allianz.pt |
| Lusitania Seguros | `lusitania` | monogram | lusitania.pt |
| Victoria Seguros | `victoria` | monogram | victoria-seguros.pt |
| Allianz | `allianz` | monogram | allianz.com, allianz.de, allianz.es, allianz.fr, ... |
| Allianz Care | `allianzcare` | monogram | allianzcare.com |
| AXA | `axa` | monogram | axa.com, axa.fr, axa.de, axa.es, ... |
| Generali | `generali` | monogram | generali.com, generali.it, generali.de, generali.fr |
| Zurich Insurance | `zurich` | monogram | zurich.com, zurich.co.uk |
| Aviva | `aviva` | monogram | aviva.com, aviva.co.uk |
| Prudential Financial | `prudential` | monogram | prudential.com |
| Prudential plc | `prudentialplc` | monogram | prudentialplc.com |
| Chubb | `chubb` | monogram | chubb.com |
| AIG | `aig` | monogram | aig.com |
| Munich Re | `munichre` | monogram | munichre.com |
| Swiss Re | `swissre` | monogram | swissre.com |
| Hannover Re | `hannoverre` | monogram | hannover-re.com |
| SCOR | `scor` | monogram | scor.com |
| Mapfre | `mapfre` | monogram | mapfre.com, mapfre.pt, mapfre.es |
| Cigna | `cigna` | monogram | cigna.com |
| Aetna | `aetna` | monogram | aetna.com |
| UnitedHealthcare | `unitedhealthcare` | monogram | uhc.com, unitedhealthcare.com, unitedhealthgroup.com |
| Elevance Health | `elevance` | monogram | elevancehealth.com, anthem.com |
| Humana | `humana` | monogram | humana.com |
| Kaiser Permanente | `kaiser` | monogram | kaiserpermanente.org, kp.org |
| Blue Cross Blue Shield | `bcbs` | monogram | bcbs.com |
| Bupa | `bupa` | monogram | bupa.com, bupa.co.uk |
| Vitality | `vitality` | monogram | vitality.co.uk, vitalityhealth.co.uk |
| Direct Line | `directline` | monogram | directline.com |
| Admiral | `admiral` | monogram | admiral.com, admiralgroup.co.uk |
| LV= | `lv` | monogram | lv.com |
| Legal & General | `legalandgeneral` | monogram | legalandgeneral.com, landg.com |
| Aegon | `aegon` | monogram | aegon.com, aegon.nl |
| NN Group | `nngroup` | monogram | nn-group.com, nn.nl |
| Achmea | `achmea` | monogram | achmea.nl |
| Talanx | `talanx` | monogram | talanx.com |
| ERGO | `ergo` | monogram | ergo.com, ergo.de |
| HDI | `hdi` | monogram | hdi.global, hdi.de |
| Covéa | `covea` | monogram | covea.eu, covea.fr |
| Groupama | `groupama` | monogram | groupama.com, groupama.fr |
| MAIF | `maif` | monogram | maif.fr |
| MACIF | `macif` | monogram | macif.fr |
| Crédit Agricole Assurances | `caassurances` | monogram | ca-assurances.com |
| CNP Assurances | `cnp` | monogram | cnp.fr |
| Sampo | `sampo` | monogram | sampo.com |
| If P&C Insurance | `ifinsurance` | monogram | if.se, if.fi, if.dk, if.no |
| Tryg | `tryg` | monogram | tryg.dk, tryg.com |
| Folksam | `folksam` | monogram | folksam.se |
| Gjensidige | `gjensidige` | monogram | gjensidige.no, gjensidige.com |
| Mutua Madrileña | `mutuamadrilena` | monogram | mutua.es |
| Caser | `caser` | monogram | caser.es |
| Berkshire Hathaway | `berkshirehathaway` | monogram | berkshirehathaway.com |
| State Farm | `statefarm` | monogram | statefarm.com |
| GEICO | `geico` | monogram | geico.com |
| Progressive | `progressive` | monogram | progressive.com |
| Allstate | `allstate` | monogram | allstate.com |
| Liberty Mutual | `libertymutual` | monogram | libertymutual.com |
| Nationwide | `nationwide` | monogram | nationwide.com |
| USAA | `usaa` | monogram | usaa.com |
| Travelers | `travelers` | monogram | travelers.com |
| Farmers Insurance | `farmers` | monogram | farmers.com |
| Lemonade | `lemonade` | monogram | lemonade.com |
| Porto Seguro | `portoseguro` | monogram | portoseguro.com.br |
| Bradesco Seguros | `bradescoseguros` | monogram | bradescoseguros.com.br |
| SulAmérica | `sulamerica` | monogram | sulamerica.com.br |
| ICBC | `icbc` | monogram | icbc.com.cn |
| Ping An | `pingan` | monogram | pingan.com |
| China Life | `chinalife` | monogram | chinalife.com.cn |
| Tokio Marine | `tokiomarine` | monogram | tokiomarine.com, tokiomarinehd.com |
| Sompo | `sompo` | monogram | sompo-hd.com |
| MS&AD | `msad` | monogram | ms-ad-hd.com |
| Dai-ichi Life | `daiichilife` | monogram | dai-ichi-life.co.jp |
| Manulife | `manulife` | monogram | manulife.com |
| Sun Life | `sunlife` | monogram | sunlife.com |
| Great-West Lifeco | `greatwestlifeco` | monogram | greatwestlifeco.com |

### logistics

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| DHL | `dhl` | real mark | dhl.com, dhl.pt, dhl.de, dhl.es, ... |
| UPS | `ups` | real mark | ups.com, ups.pt |
| FedEx | `fedex` | real mark | fedex.com |
| CTT | `ctt` | monogram | ctt.pt, cttexpresso.pt, cttexpresso.com |
| DPD | `dpd` | real mark | dpd.com, dpd.pt, dpd.de, dpd.co.uk, ... |
| GLS | `gls` | monogram | gls-group.com, gls-group.eu, gls-portugal.pt, gls-spain.es |
| TNT | `tnt` | monogram | tnt.com, tnt.it |
| USPS | `usps` | real mark | usps.com |
| Royal Mail | `royalmail` | monogram | royalmail.com |
| InPost | `inpost` | monogram | inpost.pl, inpost.eu, inpost.es, inpost.pt, ... |
| MRW | `mrw` | monogram | mrw.es, mrw.pt |
| SEUR | `seur` | monogram | seur.com, seur.es |
| Chronopost | `chronopost` | monogram | chronopost.fr, chronopost.pt, chronopost.com |
| Correos | `correos` | monogram | correos.es, correosexpress.com |
| Evri | `evri` | monogram | evri.com, myhermes.co.uk |
| Geopost | `geopost` | monogram | geopost.com |
| Maersk | `maersk` | monogram | maersk.com |

### mail

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Gmail | `gmail` | real mark | gmail.com, googlemail.com |
| Outlook | `outlook` | real mark | outlook.com, hotmail.com, live.com, outlook.office.com, ... |
| iCloud | `icloud` | real mark | icloud.com, me.com, mac.com |
| Yahoo | `yahoo` | monogram | yahoo.com, ymail.com, rocketmail.com, yahoo.co.uk |
| Fastmail | `fastmail` | monogram | fastmail.com, fastmail.fm, messagingengine.com |
| Proton Mail | `proton` | real mark | proton.me, protonmail.com, pm.me, protonmail.ch |
| GMX | `gmx` | real mark | gmx.com, gmx.net, gmx.de, web.de |
| Zoho | `zoho` | real mark | zoho.com, zoho.eu, zohomail.com |
| AOL | `aol` | real mark | aol.com |
| Yandex | `yandex` | monogram | yandex.com, yandex.ru, ya.ru |
| Mail.ru | `mailru` | real mark | mail.ru, bk.ru, inbox.ru, list.ru |
| Tuta | `tuta` | real mark | tuta.com, tutanota.com, tutamail.com |

### media

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Crunchyroll | `crunchyroll` | real mark | crunchyroll.com |
| Patreon | `patreon` | real mark | patreon.com |
| OnlyFans | `onlyfans` | real mark | onlyfans.com |
| Netflix | `netflix` | real mark | netflix.com |
| Prime Video | `primevideo` | real mark | primevideo.com |
| Disney+ | `disneyplus` | monogram | disneyplus.com, disney.com |
| Spotify | `spotify` | real mark | spotify.com |
| HBO Max | `hbomax` | real mark | hbomax.com, max.com |
| Apple Music | `applemusic` | real mark | music.apple.com |
| Deezer | `deezer` | real mark | deezer.com |
| Audible | `audible` | real mark | audible.com, audible.es, audible.de, audible.co.uk, ... |
| YouTube | `youtube` | real mark | youtube.com, youtu.be |
| Packt | `packt` | real mark | packtpub.com, packt.com |
| IMDb | `imdb` | real mark | imdb.com |
| HBO | `hbo` | real mark | hbo.com |
| Paramount+ | `paramountplus` | real mark | paramountplus.com, paramount.com |
| Stremio | `stremio` | real mark | stremio.com |
| 9GAG | `9gag` | real mark | 9gag.com |
| Yelp | `yelp` | real mark | yelp.com |
| Alamy | `alamy` | real mark | alamy.com |
| Udemy | `udemy` | real mark | udemy.com |
| Medium | `medium` | real mark | medium.com |

### news

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| The New York Times | `nytimes` | monogram | nytimes.com |
| BuzzFeed | `buzzfeed` | real mark | buzzfeed.com |
| Der Spiegel | `spiegel` | real mark | spiegel.de |
| CNN | `cnn` | real mark | cnn.com |
| BBC | `bbc` | monogram | bbc.com, bbc.co.uk |
| The Guardian | `guardian` | real mark | theguardian.com, guardian.co.uk, guardiannews.com |
| Reuters | `reuters` | monogram | reuters.com, thomsonreuters.com |
| Bloomberg | `bloomberg` | monogram | bloomberg.com, bloomberg.net |
| Financial Times | `ft` | monogram | ft.com |
| Wall Street Journal | `wsj` | monogram | wsj.com |
| Washington Post | `washingtonpost` | real mark | washingtonpost.com |
| Le Monde | `lemonde` | monogram | lemonde.fr |
| El País | `elpais` | monogram | elpais.com |
| Corriere della Sera | `corriere` | monogram | corriere.it |
| Público | `publico` | monogram | publico.pt |
| Observador | `observador` | monogram | observador.pt |
| Expresso | `expresso` | monogram | expresso.pt |
| Diário de Notícias | `dn` | monogram | dn.pt |
| Jornal de Notícias | `jn` | monogram | jn.pt |
| Correio da Manhã | `cm` | monogram | cmjornal.pt |
| SIC | `sic` | monogram | sic.pt, sicnoticias.pt |
| TVI | `tvi` | monogram | tvi.iol.pt, iol.pt |
| RTP | `rtp` | monogram | rtp.pt |
| SAPO | `sapo` | monogram | sapo.pt |
| Notícias ao Minuto | `noticiasaominuto` | monogram | noticiasaominuto.com |
| Eurosport | `eurosport` | monogram | eurosport.com, eurosport.pt |
| Euronews | `euronews` | monogram | euronews.com |
| The Economist | `economist` | monogram | economist.com |
| Le Figaro | `lefigaro` | monogram | lefigaro.fr |
| El Mundo | `elmundo` | monogram | elmundo.es |
| la Repubblica | `repubblica` | monogram | repubblica.it |
| Deutsche Welle | `dw` | real mark | dw.com |
| Die Zeit | `zeit` | monogram | zeit.de |
| Bild | `bild` | monogram | bild.de |
| Jornal de Negócios | `jornaldenegocios` | monogram | jornaldenegocios.pt |
| Record | `record` | monogram | record.pt |
| A Bola | `abola` | monogram | abola.pt |

### payments

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Adyen | `adyen` | real mark | adyen.com |
| Worldpay | `worldpay` | monogram | worldpay.com |
| Checkout.com | `checkoutcom` | monogram | checkout.com |
| Block | `block` | real mark | block.xyz, squareup.com |
| Braintree | `braintree` | real mark | braintreepayments.com, braintreegateway.com |
| Authorize.net | `authorizenet` | monogram | authorize.net |
| Mollie | `mollie` | monogram | mollie.com |
| SumUp | `sumup` | real mark | sumup.com |
| PayU | `payu` | monogram | payu.com, payu.in, payu.pl |
| Paysafe | `paysafe` | real mark | paysafe.com, paysafecard.com |
| Nuvei | `nuvei` | monogram | nuvei.com |
| Rapyd | `rapyd` | monogram | rapyd.net |
| Ifthenpay | `ifthenpay` | monogram | ifthenpay.com |
| EuPago | `eupago` | monogram | eupago.pt |
| SIBS | `sibs` | monogram | sibs.com, sibs.pt |
| Easypay | `easypay` | monogram | easypay.pt |
| Pagar.me | `pagarme` | monogram | pagar.me |
| Razorpay | `razorpay` | real mark | razorpay.com |
| Alipay | `alipay` | real mark | alipay.com |
| WeChat Pay | `wechatpay` | real mark | wechat.com, weixin.qq.com |
| UnionPay | `unionpay` | monogram | unionpay.com |
| Apple Pay | `applepay` | real mark |  |
| Google Pay | `googlepay` | real mark | pay.google.com |
| Samsung Pay | `samsungpay` | real mark |  |

### pharma

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Merck | `merck` | real mark | merckgroup.com, emdgroup.com |
| MSD | `msd` | monogram | msd.com, merck.com |
| Pfizer | `pfizer` | monogram | pfizer.com |
| Novartis | `novartis` | monogram | novartis.com |
| Roche | `roche` | monogram | roche.com |
| Sanofi | `sanofi` | monogram | sanofi.com |
| AstraZeneca | `astrazeneca` | monogram | astrazeneca.com |
| Bayer | `bayer` | monogram | bayer.com |
| GSK | `gsk` | real mark | gsk.com |
| Novo Nordisk | `novonordisk` | monogram | novonordisk.com |

### productivity

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Microsoft 365 | `m365` | monogram | office365.com, office.com, microsoft365.com, sharepoint.com, ... |
| Mozilla | `mozilla` | real mark | mozilla.org, mozilla.com, firefox.com |
| Google Workspace | `googleworkspace` | monogram | workspace.google.com |
| Adobe | `adobe` | real mark | adobe.com, adobe.io, adobesign.com |
| HubSpot | `hubspot` | real mark | hubspot.com, hubspotemail.net, hs-sites.com |
| Salesforce | `salesforce` | real mark | salesforce.com, force.com |
| Mailchimp | `mailchimp` | real mark | mailchimp.com, mcsv.net, mcdlv.net, rsgsv.net |
| Zendesk | `zendesk` | real mark | zendesk.com |
| Intercom | `intercom` | real mark | intercom.com, intercom.io, intercom-mail.com |
| DocuSign | `docusign` | real mark | docusign.com, docusign.net |
| Calendly | `calendly` | real mark | calendly.com |
| Asana | `asana` | real mark | asana.com |
| Monday.com | `monday` | monogram | monday.com |
| SAP | `sap` | real mark | sap.com |
| Oracle | `oracle` | real mark | oracle.com |
| Intuit | `intuit` | real mark | intuit.com, quickbooks.com |
| Figma | `figma` | real mark | figma.com |
| Atlassian | `atlassian` | real mark | atlassian.com |
| Bitbucket | `bitbucket` | real mark | bitbucket.org |
| Substack | `substack` | real mark | substack.com |
| Google Docs | `googledocs` | real mark | docs.google.com |
| Google Meet | `googlemeet` | real mark | meet.google.com |
| Google Play | `googleplay` | real mark | play.google.com |
| Airtable | `airtable` | real mark | airtable.com |
| Baserow | `baserow` | real mark | baserow.io |
| Ghost | `ghost` | real mark | ghost.org, ghost.io |
| Bitly | `bitly` | real mark | bitly.com, bit.ly |
| Gleam.io | `gleamio` | monogram | gleam.io |
| Databox | `databox` | monogram | databox.com |
| Prezi | `prezi` | real mark | prezi.com |
| Wix | `wix` | real mark | wix.com |
| Squarespace | `squarespace` | real mark | squarespace.com |
| Twilio SendGrid | `sendgrid` | real mark | sendgrid.com, sendgrid.net |
| Zoom | `zoom` | real mark | zoom.us, zoom.com |
| Skype | `skype` | real mark | skype.com |
| Grammarly | `grammarly` | real mark | grammarly.com |

### professional

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Accenture | `accenture` | real mark | accenture.com |
| PwC | `pwc` | monogram | pwc.com |
| Mazars | `mazars` | monogram | mazars.com, forvismazars.com |
| Deloitte | `deloitte` | monogram | deloitte.com |
| EY | `ey` | monogram | ey.com |
| KPMG | `kpmg` | monogram | kpmg.com |
| McKinsey & Company | `mckinsey` | monogram | mckinsey.com |
| BCG | `bcg` | monogram | bcg.com |
| Bain & Company | `bain` | monogram | bain.com |
| Capgemini | `capgemini` | monogram | capgemini.com |
| Cognizant | `cognizant` | real mark | cognizant.com |
| Infosys | `infosys` | real mark | infosys.com |
| Tata Consultancy Services | `tcs` | real mark | tcs.com |
| Wipro | `wipro` | monogram | wipro.com |
| Booz Allen | `boozallen` | monogram | boozallen.com |
| Roland Berger | `rolandberger` | monogram | rolandberger.com |
| Oliver Wyman | `oliverwyman` | monogram | oliverwyman.com |
| Mercer | `mercer` | monogram | mercer.com |
| Aon | `aon` | monogram | aon.com |
| Willis Towers Watson | `wtw` | monogram | wtwco.com, willistowerswatson.com |

### push

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Firebase | `firebase` | real mark | firebase.google.com, fcm.googleapis.com, firebaseio.com |
| UnifiedPush | `unifiedpush` | monogram | unifiedpush.org |

### security

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| reCAPTCHA | `recaptcha` | monogram | recaptcha.net |
| hCaptcha | `hcaptcha` | monogram | hcaptcha.com |
| Twilio | `twilio` | real mark | twilio.com |
| 1Password | `onepassword` | real mark | 1password.com |
| LastPass | `lastpass` | real mark | lastpass.com |
| Bitwarden | `bitwarden` | real mark | bitwarden.com |
| NordVPN | `nordvpn` | real mark | nordvpn.com, nordaccount.com |
| ExpressVPN | `expressvpn` | real mark | expressvpn.com |
| Dashlane | `dashlane` | real mark | dashlane.com |

### shopping

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Amazon | `amazon` | real mark | amazon.com, amazon.es, amazon.de, amazon.co.uk, ... |
| Amazon Business | `amazonbusiness` | monogram | business.amazon.com, business.amazon.es, business.amazon.de, business.amazon.co.uk, ... |
| Temu | `temu` | monogram | temu.com |
| eBay | `ebay` | real mark | ebay.com, ebay.es, ebay.de, ebay.co.uk, ... |
| Shein | `shein` | monogram | shein.com, shein.es, shein.fr, shein.de |
| Etsy | `etsy` | real mark | etsy.com |
| Shopify | `shopify` | real mark | shopify.com, myshopify.com |
| Walmart | `walmart` | real mark | walmart.com |
| IKEA | `ikea` | real mark | ikea.com, ikea.pt, ikea.es |
| Zalando | `zalando` | real mark | zalando.com, zalando.pt, zalando.es, zalando.de, ... |
| Vinted | `vinted` | real mark | vinted.com, vinted.pt, vinted.es, vinted.fr, ... |
| OLX | `olx` | monogram | olx.pt, olx.com, olx.pl, olx.ro, ... |
| Worten | `worten` | monogram | worten.pt, worten.es |
| Fnac | `fnac` | real mark | fnac.pt, fnac.com, fnac.es |
| Auchan | `auchan` | real mark | auchan.pt, auchan.fr, auchan.com |
| Lidl | `lidl` | real mark | lidl.pt, lidl.com, lidl.es, lidl.de |
| Pingo Doce | `pingodoce` | monogram | pingodoce.pt |
| Decathlon | `decathlon` | monogram | decathlon.pt, decathlon.com, decathlon.es, decathlon.fr |
| Leroy Merlin | `leroymerlin` | real mark | leroymerlin.pt, leroymerlin.es, leroymerlin.fr |
| Intermarché | `intermarche` | real mark | intermarche.com, intermarche.pt |
| MediaMarkt | `mediamarkt` | real mark | mediamarkt.com, mediamarkt.pt, mediamarkt.de, mediamarkt.es |
| Continente | `continente` | real mark | continente.pt |
| Sonae | `sonae` | monogram | sonae.pt, sonae.com |
| Gumtree | `gumtree` | real mark | gumtree.com |
| Jerónimo Martins | `jeronimomartins` | monogram | jeronimomartins.com, jeronimomartins.pt |
| Target | `target` | real mark | target.com |
| Costco | `costco` | monogram | costco.com |
| Best Buy | `bestbuy` | monogram | bestbuy.com |
| The Home Depot | `homedepot` | monogram | homedepot.com |
| AliExpress | `aliexpress` | real mark | aliexpress.com |
| Mercado Libre | `mercadolibre` | monogram | mercadolibre.com, mercadolivre.com.br |
| idealista | `idealista` | monogram | idealista.pt, idealista.com |
| Imovirtual | `imovirtual` | monogram | imovirtual.com |
| KuantoKusta | `kuantokusta` | monogram | kuantokusta.pt |
| Carrefour | `carrefour` | real mark | carrefour.com, carrefour.fr, carrefour.es |
| Tesco | `tesco` | real mark | tesco.com |

### social

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| X | `x` | real mark | x.com, twitter.com, t.co |
| WhatsApp | `whatsapp` | real mark | whatsapp.com, whatsapp.net |
| TikTok | `tiktok` | real mark | tiktok.com, tiktokv.com |
| Bing | `bing` | real mark | bing.com |
| Instagram | `instagram` | real mark | instagram.com |
| MSN | `msn` | monogram | msn.com |
| WordPress | `wordpress` | real mark | wordpress.com, wordpress.org |
| Naver | `naver` | real mark | naver.com, naver.net |
| Pinterest | `pinterest` | real mark | pinterest.com, pinterest.pt, pinterest.es, pinterest.fr, ... |
| Twitch | `twitch` | real mark | twitch.tv |
| VK | `vk` | real mark | vk.com, vk.ru |
| Canva | `canva` | real mark | canva.com |
| Samsung | `samsung` | real mark | samsung.com, samsungapps.com |
| Globo | `globo` | monogram | globo.com |
| Quora | `quora` | real mark | quora.com |
| LINE | `line` | real mark | line.me |
| Baidu | `baidu` | real mark | baidu.com |
| Signal | `signal` | real mark | signal.org |
| Snapchat | `snapchat` | real mark | snapchat.com, snap.com |
| Meta | `meta` | real mark | meta.com, fb.com |
| Facebook | `facebook` | real mark | facebook.com, facebookmail.com |
| Threads | `threads` | real mark | threads.net, threads.com |
| Reddit | `reddit` | real mark | reddit.com, redditmail.com |
| LinkedIn | `linkedin` | real mark | linkedin.com |

### storage

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Google Drive | `googledrive` | real mark | drive.google.com |
| Dropbox | `dropbox` | real mark | dropbox.com, dropboxapi.com |
| OneDrive | `onedrive` | real mark | onedrive.live.com, 1drv.ms |
| Nextcloud | `nextcloud` | real mark | nextcloud.com |
| Box | `box` | real mark | box.com, boxcloud.com |
| Amazon S3 | `s3` | real mark |  |
| Backblaze | `backblaze` | real mark | backblaze.com, backblazeb2.com |
| Wasabi | `wasabi` | real mark | wasabisys.com |
| MinIO | `minio` | real mark | min.io |

### tech

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| ABB | `abb` | real mark | abb.com |
| Datadog | `datadog` | real mark | datadoghq.com, datadoghq.eu |
| Lenovo | `lenovo` | real mark | lenovo.com |
| HP | `hp` | real mark | hp.com |
| HPE | `hpe` | monogram | hpe.com |
| Aruba | `aruba` | monogram | arubanetworks.com |
| IBM | `ibm` | real mark | ibm.com |
| Schneider Electric | `schneiderelectric` | real mark | se.com, schneider-electric.com |
| Corsair | `corsair` | real mark | corsair.com |
| Kingston Technology | `kingston` | real mark | kingston.com |
| Bose | `bose` | real mark | bose.com |
| Yamaha | `yamaha` | real mark | yamaha.com |
| Kyocera | `kyocera` | real mark | kyocera.com |
| Hitachi | `hitachi` | real mark | hitachi.com |
| LG | `lg` | real mark | lg.com |
| Dell | `dell` | real mark | dell.com |
| Dell EMC | `dellemc` | monogram | dellemc.com, emc.com, delltechnologies.com |
| ASUS | `asus` | real mark | asus.com |
| Fujitsu | `fujitsu` | real mark | fujitsu.com |
| MSI | `msi` | monogram | msi.com |
| Intel | `intel` | real mark | intel.com |
| Cisco | `cisco` | real mark | cisco.com, webex.com |
| Arista Networks | `arista` | monogram | arista.com |
| Juniper Networks | `juniper` | real mark | juniper.net |
| SolarWinds | `solarwinds` | monogram | solarwinds.com |
| F5 | `f5` | real mark | f5.com |
| MikroTik | `mikrotik` | real mark | mikrotik.com |
| Equinix | `equinix` | monogram | equinix.com |
| General Electric | `generalelectric` | real mark | ge.com |
| Hurricane Electric | `hurricaneelectric` | monogram | he.net |
| Veritas | `veritas` | real mark | veritas.com |
| Sentry | `sentry` | real mark | sentry.io |
| Wolfram | `wolfram` | real mark | wolfram.com, wolframalpha.com |
| Palantir | `palantir` | real mark | palantir.com |
| Alibaba | `alibaba` | real mark | alibaba.com, alibabacloud.com, aliyun.com |
| AnyDesk | `anydesk` | real mark | anydesk.com |
| TeamViewer | `teamviewer` | real mark | teamviewer.com |
| Gleam | `gleam` | real mark | gleam.run |
| SpaceX | `spacex` | real mark | spacex.com, starlink.com |
| Ring | `ring` | real mark | ring.com |

### telecom

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| NOS | `nos` | monogram | nos.pt |
| MEO | `meo` | monogram | meo.pt |
| Vodafone | `vodafone` | real mark | vodafone.pt, vodafone.com, vodafone.es, vodafone.de, ... |
| NOWO | `nowo` | monogram | nowo.pt |
| Altice | `altice` | monogram | altice.pt, alticeusa.com |
| Telefónica | `telefonica` | real mark | telefonica.com, movistar.es |
| Orange | `orange` | real mark | orange.com, orange.fr, orange.es |
| Deutsche Telekom | `telekom` | real mark | telekom.de, telekom.com, t-online.de |
| T-Mobile | `tmobile` | real mark | t-mobile.com |
| Verizon | `verizon` | real mark | verizon.com, verizon.net |
| AT&T | `att` | real mark | att.com, att.net |
| Sky | `sky` | real mark | sky.com, sky.it, sky.de |
| BT | `bt` | real mark | bt.com, btinternet.com |
| Comcast | `comcast` | monogram | comcast.com, comcast.net, xfinity.com |

### travel

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Booking.com | `bookingdotcom` | real mark | booking.com |
| Airbnb | `airbnb` | real mark | airbnb.com, airbnb.pt, airbnb.es, airbnb.co.uk, ... |
| Uber | `uber` | real mark | uber.com |
| Uber Eats | `ubereats` | real mark | ubereats.com |
| Ryanair | `ryanair` | real mark | ryanair.com |
| Lufthansa | `lufthansa` | real mark | lufthansa.com, lufthansa-group.com |
| TAP Air Portugal | `tap` | monogram | tap.pt, flytap.com, tapairportugal.com |
| easyJet | `easyjet` | real mark | easyjet.com |
| Air France | `airfrance` | real mark | airfrance.com, airfrance.fr, airfrance.pt |
| Emirates | `emirates` | real mark | emirates.com |
| Qatar Airways | `qatarairways` | real mark | qatarairways.com |
| Expedia | `expedia` | real mark | expedia.com, expediamail.com |
| Tripadvisor | `tripadvisor` | real mark | tripadvisor.com, tripadvisor.pt, tripadvisor.es |
| Hotels.com | `hotelscom` | real mark | hotels.com |
| FlixBus | `flixbus` | monogram | flixbus.com, flixbus.pt, flixbus.es, flixbus.de |
| Trainline | `trainline` | monogram | thetrainline.com, trainline.com, trainline.eu |
| CP Comboios de Portugal | `cp` | monogram | cp.pt |
| BlaBlaCar | `blablacar` | monogram | blablacar.com, blablacar.pt, blablacar.es, blablacar.fr |
| Via Verde | `viaverde` | monogram | viaverde.pt, via-verde.pt |
| Brisa | `brisa` | monogram | brisa.pt |
| Bolt | `bolt` | monogram | bolt.eu |
| Bolt Food | `boltfood` | monogram | food.bolt.eu |
| Glovo | `glovo` | real mark | glovoapp.com |
| Waze | `waze` | real mark | waze.com |
| KLM | `klm` | real mark | klm.com |
| Japan Airlines | `jal` | real mark | jal.com, jal.co.jp |
| Air Canada | `aircanada` | real mark | aircanada.com |
| Southwest Airlines | `southwest` | real mark | southwest.com, iflyswa.com |
| Delta Air Lines | `delta` | real mark | delta.com |
| American Airlines | `americanairlines` | real mark | aa.com, americanairlines.com |
| United Airlines | `united` | real mark | united.com, unitedairlines.com |
| China Southern Airlines | `chinasouthern` | real mark | csair.com |
| Aeroméxico | `aeromexico` | monogram | aeromexico.com |
| British Airways | `britishairways` | real mark | ba.com, britishairways.com |
| Iberia | `iberia` | real mark | iberia.com |
| Vueling | `vueling` | monogram | vueling.com |
| Wizz Air | `wizzair` | real mark | wizzair.com |
| Turkish Airlines | `turkishairlines` | real mark | turkishairlines.com, thy.com |
| Singapore Airlines | `singaporeair` | real mark | singaporeair.com |
| ANA | `ana` | real mark | ana.co.jp |
| SWISS | `swiss` | monogram | swiss.com |
| Ritz-Carlton | `ritzcarlton` | monogram | ritzcarlton.com |
| Marriott | `marriott` | real mark | marriott.com |
| Hilton | `hilton` | real mark | hilton.com |
| Hyatt | `hyatt` | monogram | hyatt.com |
| Accor | `accor` | monogram | accor.com, accorhotels.com, all.accor.com |
| IHG | `ihg` | monogram | ihg.com |
| Ticketmaster | `ticketmaster` | real mark | ticketmaster.com, ticketmaster.co.uk |
| Eventbrite | `eventbrite` | real mark | eventbrite.com |

### utility

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| Galp | `galp` | monogram | galp.com, galp.pt |
| Endesa | `endesa` | monogram | endesa.pt, endesa.com, endesa.es |
| Águas de Portugal | `aguasdeportugal` | monogram | adp.pt, aguasdeportugal.pt |
| EPAL | `epal` | monogram | epal.pt |
| ePortugal | `eportugal` | monogram | eportugal.gov.pt |
| IRN | `irn` | monogram | irn.mj.pt, irn.justica.gov.pt |
| República Portuguesa | `govpt` | monogram | gov.pt |
| DECO PROTESTE | `deco` | monogram | deco.proteste.pt, deco.pt, proteste.pt |
| Rotary International | `rotary` | real mark | rotary.org |
| EDP | `edp` | monogram | edp.com, edp.pt |
| REN | `ren` | monogram | ren.pt |
| Iberdrola | `iberdrola` | monogram | iberdrola.com, iberdrola.es |
| Repsol | `repsol` | monogram | repsol.com |
| Shell | `shell` | real mark | shell.com |
| BP | `bp` | monogram | bp.com |
| TotalEnergies | `totalenergies` | monogram | totalenergies.com |
| Engie | `engie` | monogram | engie.com, engie.fr |
| Enel | `enel` | monogram | enel.com, enel.it |
| Autoridade Tributária e Aduaneira | `autoridadetributaria` | monogram | at.gov.pt, portaldasfinancas.gov.pt |
| Segurança Social | `segurancasocial` | monogram | seg-social.pt |
| Serviço Nacional de Saúde | `sns` | monogram | sns.gov.pt, sns24.gov.pt |

