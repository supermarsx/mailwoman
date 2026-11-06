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

## Coverage

184 real marks, 74 monograms, 258 brands. Brands shown as a monogram are the candidates for a supplied official SVG:

Yahoo, Fastmail, Yandex, Linode, UnifiedPush, reCAPTCHA, hCaptcha, MSN, Globo, Amazon Business, Temu, Shein, OLX, Worten, Pingo Doce, Decathlon, Microsoft 365, Google Workspace, Monday.com, Gameforge, Kakao Games, Bandai Namco, ZeniMax, LEGO, Disney+, Santander, Caixa Geral de Depósitos, MB WAY, ActivoBank, Banco BPI, Banco CTT, Novo Banco, Trade Republic, Kraken, Skrill, Monese, Starling Bank, Chime, eToro, Millennium bcp, Montepio, Crédito Agrícola, Bankinter, ING, BNP Paribas, Lloyds Bank, Citi, CTT, GLS, TNT, Royal Mail, InPost, MRW, SEUR, Chronopost, Correos, Evri, Geopost, Maersk, TAP Air Portugal, FlixBus, Trainline, CP Comboios de Portugal, BlaBlaCar, Via Verde, Brisa, Galp, Endesa, Águas de Portugal, EPAL, ePortugal, IRN, República Portuguesa, DECO PROTESTE.

### ai

| Brand | Id | Logo | Sender domains |
| --- | --- | --- | --- |
| OpenAI | `openai` | real mark | openai.com |
| Anthropic | `anthropic` | real mark | anthropic.com |
| Google Gemini | `gemini` | real mark | generativelanguage.googleapis.com, gemini.google.com |
| Mistral AI | `mistral` | real mark | mistral.ai |
| Ollama | `ollama` | real mark | ollama.com, ollama.ai |
| Perplexity | `perplexity` | real mark | perplexity.ai |

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
| Santander | `santander` | monogram | santander.pt, santander.com, santander.es, santander.co.uk, ... |
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
| Cash App | `cashapp` | real mark | cash.app, square.com, squareup.com |
| Venmo | `venmo` | real mark | venmo.com |
| Coinbase | `coinbase` | real mark | coinbase.com |
| Binance | `binance` | real mark | binance.com |
| Kraken | `kraken` | monogram | kraken.com |
| Payoneer | `payoneer` | real mark | payoneer.com |
| Skrill | `skrill` | monogram | skrill.com |
| Monese | `monese` | monogram | monese.com |
| bunq | `bunq` | real mark | bunq.com |
| Starling Bank | `starling` | monogram | starlingbank.com |
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
| Chase | `chase` | real mark | chase.com, jpmorgan.com, jpmorganchase.com |
| Citi | `citi` | monogram | citi.com, citibank.com |
| Bank of America | `bankofamerica` | real mark | bankofamerica.com, bofa.com |

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
| Sony | `sony` | real mark | sony.com, playstation.com, sonyentertainmentnetwork.com |
| Riot Games | `riotgames` | real mark | riotgames.com |
| ZeniMax | `zenimax` | monogram | zenimax.com, bethesda.net |
| Epic Games | `epicgames` | real mark | epicgames.com |
| Nintendo | `nintendo` | real mark | nintendo.com, nintendo.net |
| LEGO | `lego` | monogram | lego.com |

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

