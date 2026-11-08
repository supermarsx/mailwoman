import {describe, expect, it} from 'vitest';
import {render, screen, waitFor} from '@solidjs/testing-library';
import {existsSync, readdirSync, readFileSync} from 'node:fs';
import {BRANDS, PROVIDER_CATEGORIES, brandFor, brandForHost, brandInText, loadBrandPath} from './registry.ts';
import {LOADERS} from './loaders.generated.ts';
import {sanitizeSvg} from '../../scripts/brands/sanitize.mjs';
import {BrandLogo, monogramHue, monogramLetter} from './BrandLogo.tsx';

const PATHS: Record<string, string> = {};
await Promise.all(Object.values(LOADERS).map(async load => Object.assign(PATHS, (await load()).PATHS)));

describe('brand registry', () => {
  it('resolves every id, name and alias to its entry; marks load per category, noMark entries have none', async () => {
    const ids = new Set<string>();
    for (const entry of BRANDS) {
      expect(ids.has(entry.id), entry.id).toBe(false);
      ids.add(entry.id);
      expect(brandFor(entry.id)?.id).toBe(entry.id);
      expect(brandFor(entry.name)?.id).toBe(entry.id);
      for (const alias of entry.aliases) expect(brandFor(alias), alias).toBeDefined();
      if (entry.noMark) {
        expect(PATHS[entry.id], entry.id).toBeUndefined();
        expect(await loadBrandPath(entry.id)).toBeUndefined();
      } else {
        expect(typeof PATHS[entry.id], entry.id).toBe('string');
        expect(await loadBrandPath(entry.id)).toBe(PATHS[entry.id]);
      }
    }
    expect(Object.keys(PATHS).sort()).toEqual(BRANDS.filter(b => !b.noMark).map(b => b.id).sort());
  });

  it('matches domains and hosts by suffix, longest first', () => {
    expect(brandFor('gmail.com')?.id).toBe('gmail');
    expect(brandFor('Google Mail')?.id).toBe('gmail');
    expect(brandFor('outlook.com')?.id).toBe('outlook');
    expect(brandForHost('imap.gmail.com')?.id).toBe('gmail');
    expect(brandForHost('https://s3.eu-west-1.amazonaws.com/bucket')?.id).toBe('aws');
    expect(brandForHost('someone@icloud.com')?.id).toBe('icloud');
    expect(brandForHost('mail.example.org:993')).toBeUndefined();
    expect(brandForHost('')).toBeUndefined();
    expect(brandForHost('com')).toBeUndefined();
    expect(brandForHost('notgmail.com')).toBeUndefined();
  });

  it('holds sanitised path data only', () => {
    for (const [id, d] of Object.entries(PATHS)) {
      expect(d, id).toMatch(/^[MmLlHhVvCcSsQqTtAaZz0-9eE.,\s+-]+$/);
      expect(d.length, id).toBeLessThan(8000);
    }
    const text = JSON.stringify(BRANDS) + JSON.stringify(PATHS);
    expect(text).not.toMatch(/https?:|\/\/|<script|<\/|javascript:|\bon[a-z]+\s*=|url\(|data:/i);
    for (const entry of BRANDS) if (entry.color) expect(entry.color).toMatch(/^#[0-9A-F]{6}$/);
  });
});

describe('BrandLogo', () => {
  it('draws the bundled mark, labelled when it stands alone', async () => {
    const {container} = render(() => <BrandLogo brand="gmail.com" size={24} />);
    expect(screen.getByRole('img', {name: 'Gmail'})).toBeTruthy();
    await waitFor(() => expect(container.querySelector('path')?.getAttribute('d')).toBe(PATHS.gmail));
    expect(container.querySelector('svg')?.getAttribute('data-size')).toBe('24');
  });

  it('is hidden from assistive technology when decorative, and mono uses currentColor', async () => {
    const {container} = render(() => <BrandLogo brand="github" decorative mono />);
    const svg = container.querySelector('svg')!;
    expect(svg.getAttribute('aria-hidden')).toBe('true');
    expect(svg.getAttribute('role')).toBeNull();
    await waitFor(() => expect(container.querySelector('path')?.getAttribute('fill')).toBe('currentColor'));
  });

  it('falls back to a deterministic monogram for unknown brands', () => {
    const {container} = render(() => <BrandLogo brand="Acme Mail" />);
    expect(container.querySelector('svg')?.getAttribute('data-brand')).toBe('monogram');
    expect(container.querySelector('text')?.textContent).toBe('A');
    expect(container.querySelector('path')).toBeNull();
    expect(monogramHue('Acme Mail')).toBe(monogramHue('acme mail'));
    expect(monogramLetter('  ünder')).toBe('Ü');
    expect(monogramLetter('')).toBe('?');
  });

  it('draws a monogram, not an empty box, for known brands without a bundled mark', () => {
    const {container} = render(() => <BrandLogo brand="cgd.pt" />);
    expect(screen.getByRole('img', {name: 'Caixa Geral de Depósitos'})).toBeTruthy();
    expect(container.querySelector('text')?.textContent).toBe('C');
    expect(container.querySelector('path')).toBeNull();
  });
});

describe('brandInText', () => {
  it('finds a brand inside an operator-chosen label, and not on short words', () => {
    expect(brandInText('Acme Okta SSO')?.id).toBe('okta');
    expect(brandInText('Sign in with Microsoft Entra')?.id).toBe('microsoft');
    expect(brandInText('Google Workspace')?.id).toBe('googleworkspace');
    expect(brandInText('Mail box live')).toBeUndefined();
    expect(brandInText('Company login')).toBeUndefined();
  });
});

describe('domain matching', () => {
  it('claims every host once', () => {
    const owner = new Map<string, string>();
    for (const entry of BRANDS) for (const host of entry.hosts) {
      expect(host, host).toMatch(/^[a-z0-9]([a-z0-9-]*[a-z0-9])?(\.[a-z0-9]([a-z0-9-]*[a-z0-9])?)+$/);
      expect(owner.get(host), `${host} claimed by ${owner.get(host)} and ${entry.id}`).toBeUndefined();
      owner.set(host, entry.id);
    }
  });

  it('resolves well-known sender domains, preferring the most specific match', () => {
    const cases: Record<string, string> = {
      'amazon.co.uk': 'amazon', 'mail.amazon.de': 'amazon', 'aws.amazon.com': 'aws', 'business.amazon.com': 'amazonbusiness',
      'paypal.com': 'paypal', 'service.paypal.com': 'paypal', 'ctt.pt': 'ctt', 'dpd.pt': 'dpd', 'dpd.com': 'dpd', 'fedex.com': 'fedex',
      'ups.com': 'ups', 'ryanair.com': 'ryanair', 'tap.pt': 'tap', 'flytap.com': 'tap', 'via-verde.pt': 'viaverde', 'bpi.pt': 'bpi',
      'cgd.pt': 'cgd', 'santander.pt': 'santander', 'novobanco.pt': 'novobanco', 'activobanco.pt': 'activobank', 'bancoctt.pt': 'bancoctt',
      'mbway.pt': 'mbway', 'galp.com': 'galp', 'endesa.pt': 'endesa', 'epal.pt': 'epal', 'eportugal.gov.pt': 'eportugal', 'gov.pt': 'govpt',
      'irn.mj.pt': 'irn', 'deco.proteste.pt': 'deco', 'brisa.pt': 'brisa', 'cp.pt': 'cp', 'x.com': 'x', 'twitter.com': 'x', 'vk.ru': 'vk',
      'office365.com': 'm365', 'outlook.com': 'outlook', 'msn.com': 'msn', 'atlassian.net': 'jira',
    };
    for (const [host, id] of Object.entries(cases)) expect(brandForHost(host)?.id, host).toBe(id);
    expect(brandForHost('no-reply@mail.paypal.com')?.id).toBe('paypal');
    expect(brandForHost('https://www.dhl.de/track?x=1')?.id).toBe('dhl');
  });

  it('never matches lookalikes or anything off a label boundary', () => {
    for (const host of ['paypal.com.evil.io', 'evilpaypal.com', 'paypal.com-secure.net', 'xpaypal.com', 'paypal-com.evil.io', 'amazon.com.evil.io',
      'gov.pt.evil.net', 'notgov.pt', 'ctt.pt.example.org', 'evil-ctt.pt', 'paypal.co', 'amazon.co', 'com.paypal', 'pt', 'co.uk', 'evil.co.uk']) {
      expect(brandForHost(host), host).toBeUndefined();
    }
    expect(brandForHost('a@paypal.com.evil.io')).toBeUndefined();
    expect(brandForHost('https://paypal.com.evil.io/login')).toBeUndefined();
    expect(brandForHost('https://evil.io/paypal.com')).toBeUndefined();
    expect(brandForHost('https://evil.io/?u=@paypal.com')).toBeUndefined();
  });

  it('resolves the named brands and aliases from the product list', () => {
    for (const name of ['X', 'WhatsApp', 'Naver', 'VK', 'Amazon Business', 'Prime Video', 'Disney+', 'Microsoft 365', 'Google Workspace', 'Gemini', 'Monday.com',
      'Square Enix', 'Activision Blizzard', 'Kakao Games', 'Bandai Namco', 'Namco', 'ZeniMax', 'MB WAY', 'CTT Expresso', 'Uber Eats', 'DECO PROTESTE',
      'Caixa Geral de Depósitos', 'Millennium bcp', 'Cash App', 'American Express', 'Royal Mail', 'CP Comboios de Portugal', 'Deezer', 'Seezer', 'Twitter']) {
      expect(brandFor(name), name).toBeDefined();
    }
    expect(brandFor('Namco')?.id).toBe('bandainamco');
    expect(brandFor('Microsoft 365')?.id).toBe('m365');
  });
});

describe('custom official logos', () => {
  const ok = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><g><path d="M1 1h4v4H1z"/><circle cx="12" cy="12" r="3"/><rect x="2" y="2" width="3" height="3"/><polygon points="0,0 4,0 4,4"/></g></svg>';
  it('accepts plain geometry and returns one path', () => {
    const d = sanitizeSvg(ok);
    expect(d).toMatch(/^[MmLlHhVvCcSsQqTtAaZz0-9eE.,\s+-]+$/);
    expect(d.startsWith('M1 1h4v4H1z')).toBe(true);
  });
  it('rejects anything that could run or load content', () => {
    const wrap = (inner: string): string => `<svg viewBox="0 0 24 24">${inner}</svg>`;
    const bad = [
      wrap('<script>alert(1)</script>'), wrap('<style>*{}</style>'), wrap('<path d="M0 0" onload="x()"/>'), wrap('<path d="M0 0" onclick="x()"/>'),
      wrap('<use href="https://evil.test/a.svg#x"/>'), wrap('<image href="a.png"/>'), wrap('<foreignObject><div/></foreignObject>'),
      wrap('<a href="javascript:x"><path d="M0 0z"/></a>'), wrap('<path d="M0 0z" style="fill:url(https://x)"/>'), wrap('<path d="M0 0z" transform="scale(2)"/>'),
      wrap('<path d="M0 0z" fill="red"/>'), wrap('<path d="url(#a)"/>'), wrap('<path d="M0 0z"/><!-- c -->'), '<!DOCTYPE svg><svg viewBox="0 0 24 24"><path d="M0 0z"/></svg>',
      '<svg viewBox="0 0 24 24"><path d="M0 0z"/>text</svg>', '<svg viewBox="0 0 48 48"><path d="M0 0z"/></svg>', '<svg viewBox="0 0 24 24"></svg>', 'not svg',
      wrap('<path d="M0 0z"/>&#x3c;'),
    ];
    for (const svg of bad) expect(() => sanitizeSvg(svg), svg).toThrow();
  });
  it('requires a source and licence in the manifest for every supplied file', () => {
    const dir = new URL('./custom/', import.meta.url);
    if (!existsSync(dir)) return;
    const manifest = existsSync(new URL('manifest.json', dir)) ? JSON.parse(readFileSync(new URL('manifest.json', dir), 'utf8')) : {};
    for (const file of readdirSync(dir).filter(f => f.endsWith('.svg'))) {
      const id = file.slice(0, -4);
      expect(brandFor(id)?.id, id).toBe(id);
      expect(String(manifest[id]?.source ?? '').trim(), id).not.toBe('');
      expect(String(manifest[id]?.licence ?? '').trim(), id).not.toBe('');
      expect(() => sanitizeSvg(readFileSync(new URL(file, dir), 'utf8')), id).not.toThrow();
    }
  });
});

describe('third batch: interpretations, domains and lookalikes', () => {
  it('resolves the typo-corrected names the product list used', () => {
    const cases: Record<string, string> = {
      abibaba: 'alibaba', Alibaba: 'alibaba', gogcom: 'gog', 'GOG.com': 'gog', Gleam: 'gleam', 'gleam.io': 'gleamio', paxb: 'pax8', Pax8: 'pax8',
      vvalley: 'vvalley', 'V-Valley': 'vvalley', 'arrows ecs': 'arrowecs', 'Arrow ECS': 'arrowecs', teasmpeak: 'teamspeak', 'theter ush': 'tether',
      blockahincom: 'blockchaincom', 'Blockchain.com': 'blockchaincom', weromexico: 'aeromexico', 'Aeroméxico': 'aeromexico', also: 'also',
      commerbank: 'commerzbank', Commerzbank: 'commerzbank', opencode: 'opencode', ring: 'ring', 'Z.ai': 'zai', cuf: 'cuf', CUF: 'cuf',
      psn: 'playstation', 'Westcom': 'comstor', 'Dell EMC': 'dellemc', 'Let\'s Encrypt': 'letsencrypt', 'Willis Towers Watson': 'wtw', HBO: 'hbo',
    };
    for (const [name, id] of Object.entries(cases)) {
      expect(brandFor(name)?.id, name).toBe(id);
    }
    // Left out on purpose: names that cannot be resolved without guessing.
    for (const name of ['mo', 'mta(uk)', 'vig consulting']) expect(brandFor(name), name).toBeUndefined();
  });

  it('matches the added sender domains by whole suffix, preferring the most specific host', () => {
    const cases: Record<string, string> = {
      'meet.google.com': 'googlemeet', 'docs.google.com': 'googledocs', 'play.google.com': 'googleplay', 'mail.google.com': 'google',
      'food.bolt.eu': 'boltfood', 'bolt.eu': 'bolt', 'mail.nytimes.com': 'nytimes', 'cmjornal.pt': 'cm', 'tvi.iol.pt': 'tvi', 'mota-engil.pt': 'motaengil',
      'edp.pt': 'edp', 'nos.pt': 'nos', 'meo.pt': 'meo', 'santandertotta.pt': 'santandertotta', 'santander.pt': 'santander', 'squareup.com': 'block',
      'cash.app': 'cashapp', 'msd.com': 'msd', 'merckgroup.com': 'merck', 'luzsaude.pt': 'luzsaude', 'cuf.pt': 'cuf', 'at.gov.pt': 'autoridadetributaria',
      'seg-social.pt': 'segurancasocial', 'eportugal.gov.pt': 'eportugal', 'other.gov.pt': 'govpt', 'playstation.com': 'playstation', 'sony.com': 'sony',
      'jpmorgan.com': 'jpmorgan', 'chase.com': 'chase', 'tdsynnex.com': 'tdsynnex', 'vodafone.pt': 'vodafone', 'hbo.com': 'hbo', 'hbomax.com': 'hbomax',
    };
    for (const [host, id] of Object.entries(cases)) expect(brandForHost(host)?.id, host).toBe(id);
  });

  it('never matches lookalikes of the added domains', () => {
    for (const host of ['nytimes.com.evil.io', 'evilnytimes.com', 'cuf.pt.example.org', 'mycuf.pt', 'edp.pt.phish.net', 'nos.pt.evil.io', 'xmeo.pt',
      'meet.google.com.evil.io', 'bolt.eu.evil.io', 'ubs.com.evil.io', 'notubs.com', 'adyen.com-pay.net', 'x.com.evil.io', 'fakex.com']) {
      expect(brandForHost(host), host).toBeUndefined();
    }
  });

  it('keeps ordinary words out of operator-chosen provider labels', () => {
    expect(brandInText('Delta', PROVIDER_CATEGORIES)).toBeUndefined();
    expect(brandInText('Target', PROVIDER_CATEGORIES)).toBeUndefined();
    expect(brandFor('Delta')?.id).toBe('delta');
    expect(brandInText('Okta', PROVIDER_CATEGORIES)?.id).toBe('okta');
    expect(brandInText('Google', PROVIDER_CATEGORIES)?.id).toBe('google');
  });
});

describe('fourth batch: insurers, Moey and eyewear', () => {
  it('resolves names, typos and domains', () => {
    const names: Record<string, string> = {
      'logo segurso': 'logoseguro', Logo: 'logoseguro', 'OK! Teleseguros': 'okseguros', 'OK Seguros': 'okseguros', 'ok-seguros': 'okseguros',
      'Ageas Portugal': 'ageas', 'Generali Tranquilidade': 'tranquilidade', 'LV=': 'lv', 'Legal & General': 'legalandgeneral', Anthem: 'elevance',
      'Blue Cross Blue Shield': 'bcbs', 'MS&AD': 'msad', 'Moey!': 'moey', 'Ray-Ban': 'rayban', 'ic! berlin': 'icberlin', Persol: 'persol', Pearle: 'pearlevision',
    };
    for (const [name, id] of Object.entries(names)) expect(brandFor(name)?.id, name).toBe(id);
    const hosts: Record<string, string> = {
      'fidelidade.pt': 'fidelidade', 'mail.ageas.pt': 'ageas', 'logo.pt': 'logoseguro', 'okteleseguros.pt': 'okseguros', 'zurich.pt': 'zurichportugal',
      'zurich.com': 'zurich', 'allianz.pt': 'allianzportugal', 'allianz.com': 'allianz', 'prudential.com': 'prudential', 'prudentialplc.com': 'prudentialplc',
      'moey.pt': 'moey', 'creditoagricola.pt': 'creditoagricola', 'ray-ban.com': 'rayban', 'luxottica.com': 'luxottica', 'essilor.com': 'essilor',
      'essilorluxottica.com': 'essilorluxottica', 'specsavers.co.uk': 'specsavers', 'ic-berlin.de': 'icberlin', 'multiopticas.pt': 'multiopticas', 'kp.org': 'kaiser',
    };
    for (const [host, id] of Object.entries(hosts)) expect(brandForHost(host)?.id, host).toBe(id);
  });

  it('rejects lookalikes of the new domains', () => {
    for (const host of ['fidelidade.pt.evil.io', 'myfidelidade.pt', 'allianz.com.phish.net', 'axa.com-login.net', 'notaxa.fr', 'moey.pt.evil.io', 'ray-ban.com.evil.io',
      'luxottica.com.evil.io', 'xlogo.pt', 'logo.pt.example.org', 'kp.org.evil.io', 'bupa.com.evil.io']) {
      expect(brandForHost(host), host).toBeUndefined();
    }
  });

  it('keeps insurers and eyewear out of provider labels, and gives every one a tile or mark', () => {
    for (const name of ['Allianz', 'Logo', 'Oakley', 'Admiral', 'Progressive']) expect(brandInText(name, PROVIDER_CATEGORIES), name).toBeUndefined();
    for (const b of BRANDS.filter(x => x.category === 'insurance' || x.category === 'eyewear')) expect(PROVIDER_CATEGORIES).not.toContain(b.category);
  });
});

describe('Microsoft tenants and MEO Empresas',()=>{
 it('maps tenant hosts to the bundled Microsoft mark and MEO enterprise hosts to MEO',async()=>{
  for(const host of ['onmicrosoft.com','tenant.onmicrosoft.com','user@tenant.onmicrosoft.com'])expect(brandForHost(host)?.id).toBe('microsoft');
  expect(await loadBrandPath('microsoft')).toBeTruthy();
  for(const host of ['meoempresas.pt','billing.meoempresas.pt'])expect(brandForHost(host)?.id).toBe('meo');
  expect(brandFor('MEO Empresas')?.id).toBe('meo');
  for(const host of ['notonmicrosoft.com','onmicrosoft.com.evil.io','tenant.onmicrosoft.com.evil.io','notmeoempresas.pt','meoempresas.pt.evil.io'])expect(brandForHost(host),host).toBeUndefined();
 });
 it('renders actual local Microsoft and MEO marks in provider logos',async()=>{
  const ms=render(()=><BrandLogo brand="tenant.onmicrosoft.com"/>).container;
  await waitFor(()=>expect(ms.querySelector('path')?.getAttribute('d')).toBe(PATHS.microsoft));
  expect(ms.querySelector('text')).toBeNull();
  const meo=render(()=><BrandLogo brand="meoempresas.pt"/>).container;
  expect(meo.querySelector('image')?.getAttribute('href')).toContain('meo-site.png');expect(meo.querySelector('text')).toBeNull();
 });
});
