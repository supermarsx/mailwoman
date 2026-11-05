// Regenerates src/brands/{registry,paths}.generated.ts from the Simple Icons dataset
// (CC0-1.0), pinned to one release. Run from apps/web: node scripts/brands/generate.mjs
// Marks missing from the pinned release are skipped and fall back to the monogram.
import { mkdirSync, writeFileSync } from 'node:fs';
// Primary release still carries Microsoft/AWS/Slack marks; newer additions come from the fallback release.
const VERSIONS = ['11.15.0', '15.22.0'];
const VERSION = VERSIONS.join(' + ');
const base = (v) => `https://raw.githubusercontent.com/simple-icons/simple-icons/${v}`;

// id, display name, simple-icons slug, aliases, hosts (domain suffixes), category
const BRANDS = [
  ['google', 'Google', 'google', ['google workspace', 'google oauth'], ['google.com', 'googleapis.com', 'accounts.google.com'], 'identity'],
  ['gmail', 'Gmail', 'gmail', ['google mail', 'googlemail'], ['gmail.com', 'googlemail.com'], 'mail'],
  ['outlook', 'Outlook', 'microsoftoutlook', ['hotmail', 'live', 'msn', 'microsoft 365', 'office 365', 'office365', 'exchange online'], ['outlook.com', 'hotmail.com', 'live.com', 'msn.com', 'office365.com', 'outlook.office.com'], 'mail'],
  ['microsoft', 'Microsoft', 'microsoft', ['entra', 'microsoft entra', 'entra id', 'azure ad', 'azure active directory'], ['microsoft.com', 'microsoftonline.com'], 'identity'],
  ['icloud', 'iCloud', 'icloud', ['icloud mail'], ['icloud.com', 'me.com', 'mac.com'], 'mail'],
  ['apple', 'Apple', 'apple', ['sign in with apple', 'apns', 'apple push'], ['apple.com', 'push.apple.com'], 'identity'],
  ['yahoo', 'Yahoo', 'yahoo', ['yahoo mail', 'ymail'], ['yahoo.com', 'ymail.com', 'rocketmail.com', 'yahoo.co.uk'], 'mail'],
  ['fastmail', 'Fastmail', 'fastmail', [], ['fastmail.com', 'fastmail.fm', 'messagingengine.com'], 'mail'],
  ['proton', 'Proton Mail', 'protonmail', ['proton', 'protonmail', 'proton mail'], ['proton.me', 'protonmail.com', 'pm.me', 'protonmail.ch'], 'mail'],
  ['gmx', 'GMX', 'gmx', [], ['gmx.com', 'gmx.net', 'gmx.de', 'web.de'], 'mail'],
  ['zoho', 'Zoho', 'zoho', ['zoho mail'], ['zoho.com', 'zoho.eu', 'zohomail.com'], 'mail'],
  ['aol', 'AOL', 'aol', [], ['aol.com'], 'mail'],
  ['yandex', 'Yandex', 'yandex', [], ['yandex.com', 'yandex.ru', 'ya.ru'], 'mail'],
  ['mailru', 'Mail.ru', 'maildotru', ['mail.ru'], ['mail.ru', 'bk.ru', 'inbox.ru', 'list.ru'], 'mail'],
  ['tuta', 'Tuta', 'tuta', ['tutanota'], ['tuta.com', 'tutanota.com', 'tutamail.com'], 'mail'],
  ['okta', 'Okta', 'okta', [], ['okta.com', 'oktapreview.com'], 'identity'],
  ['auth0', 'Auth0', 'auth0', [], ['auth0.com'], 'identity'],
  ['keycloak', 'Keycloak', 'keycloak', [], [], 'identity'],
  ['authentik', 'authentik', 'authentik', [], [], 'identity'],
  ['github', 'GitHub', 'github', [], ['github.com', 'githubusercontent.com'], 'identity'],
  ['gitlab', 'GitLab', 'gitlab', [], ['gitlab.com'], 'identity'],
  ['slack', 'Slack', 'slack', [], ['slack.com'], 'app'],
  ['teams', 'Microsoft Teams', 'microsoftteams', ['teams', 'ms teams'], ['teams.microsoft.com', 'webhook.office.com'], 'app'],
  ['discord', 'Discord', 'discord', [], ['discord.com', 'discordapp.com'], 'app'],
  ['zapier', 'Zapier', 'zapier', [], ['zapier.com'], 'app'],
  ['matrix', 'Matrix', 'matrix', ['element'], ['matrix.org'], 'app'],
  ['mattermost', 'Mattermost', 'mattermost', [], ['mattermost.com'], 'app'],
  ['telegram', 'Telegram', 'telegram', [], ['telegram.org', 't.me'], 'app'],
  ['jira', 'Jira', 'jira', [], ['atlassian.net', 'jira.com'], 'app'],
  ['notion', 'Notion', 'notion', [], ['notion.so', 'notion.com'], 'app'],
  ['trello', 'Trello', 'trello', [], ['trello.com'], 'app'],
  ['googledrive', 'Google Drive', 'googledrive', ['gdrive'], ['drive.google.com'], 'storage'],
  ['dropbox', 'Dropbox', 'dropbox', [], ['dropbox.com', 'dropboxapi.com'], 'storage'],
  ['onedrive', 'OneDrive', 'microsoftonedrive', [], ['onedrive.live.com', '1drv.ms'], 'storage'],
  ['nextcloud', 'Nextcloud', 'nextcloud', [], ['nextcloud.com'], 'storage'],
  ['box', 'Box', 'box', [], ['box.com', 'boxcloud.com'], 'storage'],
  ['aws', 'Amazon Web Services', 'amazonaws', ['aws', 'amazon aws', 'ses', 'amazon ses', 'sns'], ['amazonaws.com', 'aws.amazon.com', 'amazonses.com', 'awsapps.com'], 'cloud'],
  ['s3', 'Amazon S3', 'amazons3', ['aws s3', 's3'], [], 'storage'],
  ['googlecloud', 'Google Cloud', 'googlecloud', ['gcp', 'google cloud storage', 'gcs'], ['cloud.google.com', 'storage.googleapis.com', 'run.app', 'appspot.com'], 'cloud'],
  ['azure', 'Microsoft Azure', 'microsoftazure', ['azure blob', 'azure blob storage', 'azure openai'], ['azure.com', 'windows.net', 'openai.azure.com', 'azurewebsites.net'], 'cloud'],
  ['cloudflare', 'Cloudflare', 'cloudflare', ['cloudflare r2', 'r2', 'turnstile', 'cloudflare turnstile', 'cloudflare web analytics'], ['cloudflare.com', 'cloudflare-dns.com', 'cloudflarestorage.com', 'cloudflareinsights.com'], 'cloud'],
  ['backblaze', 'Backblaze', 'backblaze', ['backblaze b2', 'b2'], ['backblaze.com', 'backblazeb2.com'], 'storage'],
  ['wasabi', 'Wasabi', 'wasabi', [], ['wasabisys.com'], 'storage'],
  ['minio', 'MinIO', 'minio', [], ['min.io'], 'storage'],
  ['digitalocean', 'DigitalOcean', 'digitalocean', ['digitalocean spaces', 'spaces'], ['digitalocean.com', 'digitaloceanspaces.com'], 'cloud'],
  ['hetzner', 'Hetzner', 'hetzner', [], ['hetzner.com', 'hetzner.cloud', 'your-server.de', 'your-storagebox.de'], 'cloud'],
  ['ovh', 'OVHcloud', 'ovh', ['ovhcloud'], ['ovh.com', 'ovh.net', 'ovhcloud.com'], 'cloud'],
  ['vultr', 'Vultr', 'vultr', [], ['vultr.com'], 'cloud'],
  ['linode', 'Linode', 'linode', ['akamai connected cloud'], ['linode.com', 'linodeobjects.com'], 'cloud'],
  ['akamai', 'Akamai', 'akamai', [], ['akamai.com', 'akamaiedge.net'], 'cloud'],
  ['fastly', 'Fastly', 'fastly', [], ['fastly.com', 'fastly.net'], 'cloud'],
  ['plausible', 'Plausible', 'plausibleanalytics', ['plausible analytics'], ['plausible.io'], 'analytics'],
  ['matomo', 'Matomo', 'matomo', [], ['matomo.cloud', 'matomo.org'], 'analytics'],
  ['googleanalytics', 'Google Analytics', 'googleanalytics', ['ga4', 'google analytics 4'], ['google-analytics.com', 'analytics.google.com', 'googletagmanager.com'], 'analytics'],
  ['posthog', 'PostHog', 'posthog', [], ['posthog.com'], 'analytics'],
  ['umami', 'Umami', 'umami', [], ['umami.is'], 'analytics'],
  ['fathom', 'Fathom', 'fathom', ['fathom analytics'], ['usefathom.com'], 'analytics'],
  ['firebase', 'Firebase', 'firebase', ['fcm', 'firebase cloud messaging'], ['firebase.google.com', 'fcm.googleapis.com', 'firebaseio.com'], 'push'],
  ['unifiedpush', 'UnifiedPush', 'unifiedpush', [], ['unifiedpush.org'], 'push'],
  ['recaptcha', 'reCAPTCHA', 'googlerecaptcha', ['google recaptcha'], ['recaptcha.net'], 'security'],
  ['hcaptcha', 'hCaptcha', 'hcaptcha', [], ['hcaptcha.com'], 'security'],
  ['twilio', 'Twilio', 'twilio', [], ['twilio.com'], 'security'],
  ['openai', 'OpenAI', 'openai', ['chatgpt', 'gpt'], ['openai.com'], 'ai'],
  ['anthropic', 'Anthropic', 'anthropic', ['claude'], ['anthropic.com'], 'ai'],
  ['gemini', 'Google Gemini', 'googlegemini', ['gemini'], ['generativelanguage.googleapis.com', 'gemini.google.com'], 'ai'],
  ['mistral', 'Mistral AI', 'mistralai', ['mistral'], ['mistral.ai'], 'ai'],
  ['ollama', 'Ollama', 'ollama', [], ['ollama.com', 'ollama.ai'], 'ai'],
];

const hexByVersion = {};
for (const v of VERSIONS) {
  const data = await (await fetch(`${base(v)}/${v.startsWith("11") ? "_data" : "data"}/simple-icons.json`)).json();
  const m = new Map();
  for (const i of data.icons ?? data) m.set(i.slug ?? i.title.toLowerCase().replace(/[^a-z0-9]/g, ''), i.hex);
  hexByVersion[v] = m;
}

const lum = (hex) => {
  const c = [0, 2, 4].map(i => parseInt(hex.slice(i, i + 2), 16) / 255).map(v => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
};
const reg = [];
const paths = {};
const missing = [];
for (const [id, name, slug, aliases, hosts, category] of BRANDS) {
  let res;
  let used = VERSIONS[0];
  for (const v of VERSIONS) { used = v; res = await fetch(`${base(v)}/icons/${slug}.svg`); if (res.ok) break; }
  if (!res.ok) { missing.push(`${id}(${slug})`); continue; }
  const svg = await res.text();
  const all = [...svg.matchAll(/<path d="([^"]+)"/g)];
  if (all.length !== 1) { missing.push(`${id}(paths=${all.length})`); continue; }
  const hex = String(hexByVersion[used].get(slug) ?? '').toUpperCase();
  // Brand colour only when it reads on both light and dark surfaces; otherwise the mark follows the text colour.
  const L = hex ? lum(hex) : -1;
  reg.push({ id, name, aliases, hosts, category, ...(hex && L >= 0.08 && L <= 0.45 ? { color: `#${hex}` } : {}) });
  paths[id] = all[0][1];
}
const head = `// GENERATED by scripts/brands/generate.mjs from Simple Icons ${VERSION} (CC0-1.0). Do not edit.\n`;
mkdirSync(new URL('../../src/brands/', import.meta.url), { recursive: true });
writeFileSync(new URL('../../src/brands/registry.generated.ts', import.meta.url),
  `${head}import type {BrandEntry} from './types.ts';\nexport const BRANDS:readonly BrandEntry[]=${JSON.stringify(reg)};\n`);
writeFileSync(new URL('../../src/brands/paths.generated.ts', import.meta.url),
  `${head}/** Single-path marks on a 24x24 viewBox; loaded lazily, see BrandLogo. */\nexport const PATHS:Readonly<Record<string,string>>=${JSON.stringify(paths)};\n`);
console.log('ok', reg.length, 'missing:', missing.join(', '));
