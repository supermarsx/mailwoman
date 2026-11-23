# Crawler identification

Request protection sorts each request by the product name in its `User-Agent` header: a browser or mail client, a search crawler, an AI crawler, a script, an unfamiliar client, or no header at all. Some classes are refused on the first request by default. A `User-Agent` is whatever the client chooses to send: these classes describe what a request claims to be, not who sent it. A matching name never grants access and never skips sign-in, CSRF checks or any other control, and the server makes no DNS or other lookup to confirm a crawler.

The rules are on **Security centre** > **Aggregate and adaptive protection** > **Aggregate**, in the group **Browsers and crawlers**.

![The first rules of the Browsers and crawlers group: User-Agent protection set to Enforce, Block recognized bots on the first request switched on, and the categories to count.](/screenshots/admin-access/crawler-identification-user-agent.webp){.mw-crop}

## Classes and built-in names

Names are matched case-insensitively as whole words anywhere in the header. When several match, the class higher in this table wins.

| Class (console label) | Built-in names |
|---|---|
| `ai-user` (**User-triggered AI fetches**) | ChatGPT-User, Claude-User, Perplexity-User, Amzn-User, MistralAI-User, meta-externalfetcher |
| `ai-search` (**AI search crawlers**) | OAI-SearchBot, Claude-SearchBot, PerplexityBot, DuckAssistBot, Amzn-SearchBot |
| `ai-training` (**AI training crawlers**) | GPTBot, ClaudeBot, CCBot, Amazonbot, Google-CloudVertexBot, Bytespider, meta-externalagent, FacebookBot, Google-Extended, Applebot-Extended |
| `search` (**Search engines**) | Googlebot (and -Image, -News, -Video, -StoreBot), Google-InspectionTool, GoogleOther (and -Image, -Video), bingbot, AdIdxBot, Applebot, DuckDuckBot, YandexBot, YandexMobileBot, YandexComBot |
| `automation` (**Automation and custom patterns**) | curl, wget, python-requests, python-urllib, httpx, Go-http-client, HeadlessChrome, PhantomJS, Selenium, Scrapy, aiohttp, libwww-perl, Java; and any product name ending in `bot`, `spider` or `crawler` that is not listed above |
| `missing` (**Missing header**) | an empty or absent `User-Agent` |
| `uncommon` (**Uncommon browser or client**) | anything else, except a `Mozilla/5.0` header naming Chrome, Chromium, Firefox, Safari, Edge or Opera, and headers starting with Mailwoman, Thunderbird, Outlook, Microsoft Office, AppleMail or DAVx5, which are not classed |
| `custom` | headers matching **Additional User-Agent patterns** (`security.protection.userAgentPatterns`, case-insensitive regular expressions) |

Google-Extended and Applebot-Extended are names publishers use in `robots.txt` rather than in a `User-Agent`; they are in the list so that a request claiming them is handled like the other training crawlers. Headers longer than the server's limit are treated as `uncommon`.

## What happens by default

For requests without a session (and sign-in requests):

| Class | Default action | Answer |
|---|---|---|
| AI training, AI search, user-triggered AI fetches | refused on every request | `403` `automatedClientBlocked` |
| Search engines, automation, custom patterns | blocked on the first request for **Block** (`security.protection.blockSecs`, 300 s) | `429` `requestProtectionLimited` with `Retry-After` |
| Missing header, uncommon client | allowed up to 600 requests per address per 60 seconds | `429` above that |
| Browsers and mail clients | no user-agent rule | |

The first-request block applies to that class from that address only: a browser on the same address is still served. `/healthz`, `/readyz` and sign-out are not covered; static assets are covered by the refusals and blocks but not by the budgets.

Requests with a valid mailbox or console session are not refused for their user-agent class, and by default are not counted against a user-agent budget either.

## Settings

| Label | Key | Default | Effect |
|---|---|---|---|
| **User-Agent protection** | [`security.protection.userAgentMode`](/admin/settings#security.protection.userAgentMode) | Enforce | **Observe** records decisions without refusing; **Off** turns off the budgets and the first-request block, but not the `403` refusals below |
| **Block recognized bots on the first request** | [`security.protection.userAgentBlockBots`](/admin/settings#security.protection.userAgentBlockBots) | on | off: search, automation and custom claims get the crawler or weak budget instead of a block |
| **Categories to count** | `security.protection.userAgentCategories` | all seven | a class not selected gets no budget and no first-request block; its `403` refusal still applies |
| **Missing, uncommon and automation request budget** | `security.protection.userAgentWeakRequests` | 600 | per address per window |
| **Crawler request budget** | `security.protection.userAgentCrawlerRequests` | 300 | per address per window, used when the first-request block is off |
| **User-Agent counting window** | `security.protection.userAgentWindowSecs` | 60 s | |
| **Strongest User-Agent action** | `security.protection.userAgentMaxAction` | throttle | cap for repeated weak-signal escalation; the first-request block is not affected |
| **Requests covered** | `security.protection.userAgentScope` | anonymous | **all** also counts requests with a session |
| **Combine with repeated unusual requests** | `security.protection.userAgentAdaptive` | on | |
| **Additional User-Agent patterns** | `security.protection.userAgentPatterns` | empty | up to 32 regular expressions, class `custom` |
| **User-Agent exceptions** | `security.protection.userAgentExceptions` | empty | up to 32 regular expressions; a matching header is not classed at all |

The refuse-or-allow action per class is set by the `security.trust.ua*` settings: `uaAiTraining`, `uaAiSearch` and `uaAiUser` are `block` by default ([`security.trust.uaAiTraining`](/admin/settings#security.trust.uaAiTraining)), `uaMissing`, `uaUncommon`, `uaAutomation`, `uaSearch` and `uaCustom` are `limit`. `allow` skips the user-agent budget for that class. These actions apply only to requests without a session.

To let one client through, for example a monitoring agent that sends `curl`, add a pattern for its exact header to **User-Agent exceptions**. An exception exempts the request from these rules only; request budgets, adaptive protection and sign-in rules still apply.

## Related

- [Aggregate and adaptive protection](./request-protection.md)
- [Scanner-path catalog](./scan-path-catalog.md)
