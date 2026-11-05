# Brand logos

The web app shows a logo beside the name of third-party providers and apps: single sign-on providers on the sign-in page and in Admin > Single sign-on, AI assistant providers, analytics providers, webhook targets, egress destinations and the provider of a linked mailbox.

## Where the marks come from

- Path data comes from [Simple Icons](https://github.com/simple-icons/simple-icons), released under CC0-1.0 (public domain dedication). Most marks are taken from release 11.15.0; Newer additions (GMX, Tuta, Mistral AI, Ollama) come from 15.22.0, because those marks are not in the older release.
- `apps/web/scripts/brands/generate.mjs` pins both releases and writes `apps/web/src/brands/registry.generated.ts` (names, aliases, host suffixes) and `paths.generated.ts` (path data). Run it from `apps/web` to refresh.
- Marks that Simple Icons does not carry (Yahoo, Fastmail, Yandex, Linode, UnifiedPush, reCAPTCHA, hCaptcha) are not redrawn. They show a neutral letter tile instead.

## Trademarks

All names and logos are trademarks or registered trademarks of their owners. They are shown only to identify the service a setting refers to. Their presence does not imply endorsement by, or affiliation with, those companies, and the CC0 licence of the path data does not grant any trademark rights.

## Privacy and security

Logos are bundled with the app as inline SVG path data. Nothing is fetched from a CDN, favicon service or the provider itself, so showing a logo never reveals which providers an operator uses. The path data is loaded lazily as one separate chunk the first time a logo is drawn. A test checks that no entry contains a URL, script or event handler.

## Using it

```tsx
import {BrandLogo, BrandLabel, BrandCell, HostBrandCell} from '../brands/BrandLogo.tsx';
<BrandLogo brand="gmail.com" size={24} />       // standalone: role="img", labelled
<BrandLabel brand="okta" />                      // logo + visible name, logo is decorative
<BrandCell brand="github">...content...</BrandCell>
<HostBrandCell host={url}>...</HostBrandCell>   // only when the host is a known brand
```

`brand` resolves by id, display name, alias or domain. `brandForHost()` matches host names, URLs and e-mail addresses by domain suffix, and `brandInText()` finds a brand inside a free-text label. Sizes are 16, 20, 24 and 32. Single-colour marks follow the text colour unless the brand colour reads on both light and dark surfaces; `mono` forces the text colour. In forced-colors mode every logo uses system colours.
