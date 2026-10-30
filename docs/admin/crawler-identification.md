# Crawler identification reference

Reviewed 2026-10-30. These names identify **claims in a request's User-Agent**, not authenticated operators. A matching name must never grant access or bypass CSRF, authentication, outbound-request policy or message isolation. Classification performs no per-request DNS or external lookup.

## Published identifiers

| Publisher | HTTP user-agent identifiers | Source |
| --- | --- | --- |
| Google | Googlebot, Googlebot-Image, Googlebot-Video, GoogleOther, GoogleOther-Image, GoogleOther-Video, Google-CloudVertexBot | [Google crawler reference](https://developers.google.com/crawling/docs/crawlers-fetchers/google-common-crawlers) |
| Microsoft | bingbot, AdIdxBot | [Bing crawler reference](https://www.bing.com/webmasters/help/which-crawlers-does-bing-use-8c184ec0) |
| OpenAI | GPTBot, OAI-SearchBot, ChatGPT-User | [OpenAI crawler reference](https://developers.openai.com/api/docs/bots) |
| Anthropic | ClaudeBot, Claude-SearchBot, Claude-User | [Anthropic crawler reference](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler) |
| Perplexity | PerplexityBot, Perplexity-User | [Perplexity crawler reference](https://docs.perplexity.ai/docs/resources/perplexity-crawlers) |
| Amazon | Amazonbot, Amzn-SearchBot, Amzn-User | [Amazon crawler reference](https://developer.amazon.com/docs/amazonbot/amazonbot.html) |
| Apple | Applebot | [Apple crawler reference](https://support.apple.com/en-us/119829) |
| DuckDuckGo | DuckDuckBot, DuckAssistBot | [Search crawler](https://duckduckgo.com/duckduckgo-help-pages/results/duckduckbot), [AI answer crawler](https://duckduckgo.com/duckduckgo-help-pages/results/duckassistbot) |
| Common Crawl | CCBot | [Common Crawl reference](https://commoncrawl.org/ccbot) |
| Yandex | YandexBot, YandexMobileBot, YandexComBot | [Yandex crawler reference](https://yandex.com/support/webmaster/en/robot-workings/check-yandex-robots) |

Cloudflare's maintained [AI bot reference](https://developers.cloudflare.com/ai-crawl-control/reference/bots/) additionally lists Bytespider, meta-externalagent, meta-externalfetcher, FacebookBot and MistralAI-User. Meta's publisher documentation returned HTTP429 during this review; the Cloudflare reference is the source for those entries, not a claim of publisher verification.

Training collectors, AI search crawlers and user-triggered fetchers have different purposes; keep their controls separate. User-agent versions change, so matching should use product names without pinning a browser or bot version. Custom patterns remain available for operators and products not covered by the built-in catalog.

## Robots-only controls are different

Google-Extended has no separate HTTP user agent. It controls permitted uses of data collected by existing Google crawlers. Applebot-Extended similarly controls training use of Applebot-collected data. Neither should appear as a supposed independently detectable HTTP crawler. Blocking a robots.txt token is not an HTTP enforcement capability and does not authenticate a crawler.

## Conservative enforcement

An unusual or missing User-Agent is weak evidence: command-line clients, accessibility tools, embedded browsers and API integrations can all be legitimate. Matching a crawler name is also not proof of malicious intent. Use bounded request windows and configurable thresholds, preserve explicit exceptions, and keep a single unusual request from causing an immediate block. Record the matched category and reason so an administrator can explain and tune a decision.

This reference describes catalog provenance and policy design. The settings registry and policy simulator are the authority for the currently implemented defaults, categories, thresholds and actions.
