// The pre-paint boot script (t52). It is plain JavaScript that Vite does not
// bundle, so these tests run the file's own text, the way the browser does.
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { defaultAppearance, FONT_STACKS, resolveThemeName, type AppearancePrefs } from '../theme/appearance.ts';
import { THEME_REGISTRY } from '../theme/registry.ts';
import type { ThemeName } from '../theme/contract.css.ts';
import { LOCALES, negotiateLocale } from '../i18n/locales.ts';
import { BOOT_MANIFEST_ID, buildBootManifest, fontSlug, manifestTag, readManifestTag, UI_FONT_SLUGS, type BootManifest, type ManifestChunk } from './bootManifest.ts';

const SOURCE = readFileSync(join(dirname(fileURLToPath(import.meta.url)), 'boot.js'), 'utf8');
const run = (): void => { new Function(SOURCE)(); };
const root = document.documentElement;
const BOOT_ATTRS = ['data-theme', 'data-appearance', 'data-density', 'data-accent', 'data-badge', 'data-mw-boot', 'data-reduced-motion', 'data-mw-boot-logo', 'data-mw-boot-nopasskey'];

function snapshot(prefs: AppearancePrefs, theme: ThemeName, extra: Record<string, unknown> = {}): void {
  localStorage.setItem('mw.boot.theme', JSON.stringify({
    v: 1, theme, appearance: THEME_REGISTRY[theme].appearance, mode: prefs.mode, light: prefs.lightTheme, dark: prefs.darkTheme,
    schedule: prefs.schedule, density: prefs.density, accent: prefs.accentId, badge: prefs.badgeColor, font: '', props: {}, ...extra,
  }));
}

function mockDark(dark: boolean): void {
  vi.stubGlobal('matchMedia', (q: string) => ({ matches: q.includes('dark') ? dark : false, media: q }));
}

beforeEach(() => {
  localStorage.clear();
  for (const a of BOOT_ATTRS) root.removeAttribute(a);
  root.removeAttribute('style');
  document.head.querySelectorAll('meta[name="mw-console"]').forEach((m) => m.remove());
  mockDark(false);
});
afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
  localStorage.clear();
  for (const a of BOOT_ATTRS) root.removeAttribute(a);
  root.removeAttribute('style');
});

describe('boot.js', () => {
  it('leaves the CSS defaults alone with nothing stored, and draws no mailbox', () => {
    run();
    expect(root.hasAttribute('data-theme')).toBe(false);
    expect(root.getAttribute('data-mw-boot')).toBe('plain');
  });

  it('replays the last applied fixed theme, density, accent, badge and font', () => {
    const prefs: AppearancePrefs = { ...defaultAppearance(), theme: 'light', density: 'compact', accentId: 'teal' as AppearancePrefs['accentId'], badgeColor: 'accent', chosen: true };
    snapshot(prefs, 'light', { font: 'system-ui, sans-serif', props: { '--x-accent': '#123456', color: 'red' } });
    run();
    expect(root.getAttribute('data-theme')).toBe('light');
    expect(root.getAttribute('data-appearance')).toBe('light');
    expect(root.style.getPropertyValue('color-scheme')).toBe('light');
    expect(root.getAttribute('data-density')).toBe('compact');
    expect(root.getAttribute('data-accent')).toBe('teal');
    expect(root.getAttribute('data-badge')).toBe('accent');
    expect(root.style.getPropertyValue('--mw-ui-font')).toBe('system-ui, sans-serif');
    expect(root.style.getPropertyValue('--x-accent')).toBe('#123456');
    // Only custom properties are replayed from storage.
    expect(root.style.getPropertyValue('color')).toBe('');
  });

  it.each([true, false])('resolves "follow system" the way the theme layer does (OS dark: %s)', (dark) => {
    mockDark(dark);
    const prefs: AppearancePrefs = { ...defaultAppearance(), mode: 'system', lightTheme: 'light', darkTheme: 'dark', chosen: true };
    // Stored while the OS said the opposite: the custom accent was for that pack.
    snapshot(prefs, dark ? 'light' : 'dark', { props: { '--x-accent': '#123456' } });
    run();
    const expected = resolveThemeName(prefs, { systemDark: dark, now: new Date() });
    expect(root.getAttribute('data-theme')).toBe(expected);
    expect(root.getAttribute('data-appearance')).toBe(dark ? 'dark' : 'light');
    expect(root.style.getPropertyValue('--x-accent')).toBe('');
  });

  it.each(['03:00', '12:00', '21:30'])('resolves the schedule the way the theme layer does at %s', (time) => {
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(new Date(`2026-10-29T${time}:00`));
    const prefs: AppearancePrefs = { ...defaultAppearance(), mode: 'schedule', chosen: true };
    snapshot(prefs, 'dark');
    run();
    expect(root.getAttribute('data-theme')).toBe(resolveThemeName(prefs, { systemDark: false, now: new Date() }));
  });

  it('draws the mailbox only when a session was open, and not on consent pages', () => {
    localStorage.setItem('mw.boot.session', '1');
    run();
    expect(root.getAttribute('data-mw-boot')).toBe('mail');
    history.replaceState(null, '', '/oauth/authorize');
    run();
    expect(root.getAttribute('data-mw-boot')).toBe('plain');
    history.replaceState(null, '', '/');
  });

  function consolePage(): void {
    const meta = document.createElement('meta');
    meta.name = 'mw-console';
    meta.content = 'admin';
    document.head.append(meta);
  }

  it('on the admin console page draws the sign-in card and replays none of the webmail (t72)', () => {
    snapshot({ ...defaultAppearance(), theme: 'light', chosen: true }, 'light');
    localStorage.setItem('mw.boot.session', '1');
    localStorage.setItem('mw.boot.geometry', JSON.stringify({ v: 1, wide: window.innerWidth >= 761, side: 300 }));
    consolePage();
    run();
    expect(root.hasAttribute('data-theme')).toBe(false);
    expect(root.getAttribute('data-mw-boot')).toBe('admin');
    expect(root.style.getPropertyValue('--mwb-side')).toBe('');
    expect(root.style.getPropertyValue('--mwb-admin-card')).toBe('');
  });

  it('draws the console frame when an admin session was open last time (t72)', () => {
    localStorage.setItem('mw.boot.admin', '1');
    localStorage.setItem('mw.boot.admin.card', JSON.stringify({ v: 1, wide: window.innerWidth >= 761, height: 400 }));
    consolePage();
    run();
    expect(root.getAttribute('data-mw-boot')).toBe('admin-console');
    // The card height is for the card, not the frame.
    expect(root.style.getPropertyValue('--mwb-admin-card')).toBe('');
  });

  it('sizes the card as it was last measured, for the same layout class only, and clamps it (t72)', () => {
    const wide = window.innerWidth >= 761;
    consolePage();
    localStorage.setItem('mw.boot.admin.card', JSON.stringify({ v: 1, wide, height: 357.54, logo: true }));
    run();
    expect(root.style.getPropertyValue('--mwb-admin-card')).toBe('357.5px');
    expect(root.hasAttribute('data-mw-boot-logo')).toBe(true);
    for (const stored of [{ v: 1, wide: !wide, height: 357 }, { v: 1, wide, height: 99999 }, { v: 1, wide, height: 'x' }, { v: 2, wide, height: 357 }]) {
      root.removeAttribute('style');
      localStorage.setItem('mw.boot.admin.card', JSON.stringify(stored));
      run();
      expect(root.style.getPropertyValue('--mwb-admin-card'), JSON.stringify(stored)).toBe('');
    }
  });

  it('hides the passkey button placeholder where the browser has no WebAuthn (t72)', () => {
    consolePage();
    vi.stubGlobal('PublicKeyCredential', undefined);
    run();
    expect(root.hasAttribute('data-mw-boot-nopasskey')).toBe(true);
  });

  it('applies stored geometry only for the same layout class, and clamps it', () => {
    const wide = window.innerWidth >= 761;
    localStorage.setItem('mw.boot.geometry', JSON.stringify({ v: 1, wide, side: 300, tabs: 43, top: 70, list: -1, lead: 99999, row: 64 }));
    run();
    expect(root.style.getPropertyValue('--mwb-side')).toBe('300px');
    expect(root.style.getPropertyValue('--mwb-row')).toBe('64px');
    expect(root.style.getPropertyValue('--mwb-list')).toBe('');
    expect(root.style.getPropertyValue('--mwb-lead')).toBe('');
    root.removeAttribute('style');
    localStorage.setItem('mw.boot.geometry', JSON.stringify({ v: 1, wide: !wide, side: 300 }));
    run();
    expect(root.style.getPropertyValue('--mwb-side')).toBe('');
  });

  it('replays mailbox geometry at the same 760/761 breakpoint as the live shell',()=>{
    for(const [width,wide] of [[760,false],[761,true]] as const){
      vi.stubGlobal('innerWidth',width);root.removeAttribute('style');
      localStorage.setItem('mw.boot.geometry',JSON.stringify({v:1,wide,side:wide?226:0,top:70,row:72}));
      run();
      expect(root.style.getPropertyValue('--mwb-row'),String(width)).toBe('72px');
    }
  });

  it('survives corrupt storage', () => {
    localStorage.setItem('mw.boot.theme', '{not json');
    localStorage.setItem('mw.boot.geometry', '[]');
    expect(run).not.toThrow();
    expect(root.hasAttribute('data-theme')).toBe(false);
  });
});

// ── t52w: first-screen downloads from the build manifest ───────────────────
describe('boot.js first-screen downloads (t52w)', () => {
  const MANIFEST: BootManifest = {
    v: 1,
    mail: ['./assets/mail-AAAA.js', './assets/shared-BBBB.js', './assets/mail-CCCC.css'],
    login: ['./assets/login-DDDD.js', './assets/login-EEEE.css'],
    fonts: { jost: './assets/jost-latin-wght-normal-FFFF.woff2', inter: './assets/inter-latin-wght-normal-GGGG.woff2' },
    packs: Object.fromEntries(LOCALES.map((l) => [l, `./assets/catalog.${l}.app-HHHH.json`])),
    sri: { './assets/mail-AAAA.js': 'sha384-mail', './assets/login-DDDD.js': 'sha384-login' },
  };
  type Parked = Record<string, Promise<boolean>>;
  const parked = (): Parked => (window as { __mwPreloaded?: Parked }).__mwPreloaded ?? {};
  let fetched: { url: string; init: RequestInit }[];

  function page(manifest: BootManifest | string | null = MANIFEST): void {
    document.getElementById(BOOT_MANIFEST_ID)?.remove();
    document.head.querySelectorAll('link[rel="preload"]').forEach((l) => l.remove());
    delete (window as { __mwPreloaded?: unknown }).__mwPreloaded;
    if (manifest === null) return;
    const holder = document.createElement('div');
    holder.innerHTML = typeof manifest === 'string' ? manifest : manifestTag(manifest);
    document.head.append(holder.firstElementChild!);
  }
  const links = (): { as: string | null; href: string; cors: string | null }[] => [...document.head.querySelectorAll('link[rel="preload"]')]
    .map((l) => ({ as: l.getAttribute('as'), href: l.getAttribute('href')!, cors: l.getAttribute('crossorigin') }));
  const packLinks = (): string[] => links().filter((l) => l.as === 'fetch').map((l) => l.href);
  const abs = (href: string): string => new URL(href, document.baseURI).href;
  const languages = (list: readonly string[]): void => {
    vi.spyOn(navigator, 'languages', 'get').mockReturnValue(list);
    vi.spyOn(navigator, 'language', 'get').mockReturnValue(list[0] ?? 'en');
  };

  beforeEach(() => {
    fetched = [];
    vi.stubGlobal('fetch', (url: string, init: RequestInit) => { fetched.push({ url, init }); return Promise.resolve(new Response('x')); });
    languages(['en-US']);
  });
  afterEach(() => { page(null); history.replaceState(null, '', '/'); vi.restoreAllMocks(); });

  it('with a session last time: fetches the mailbox files (scripts with SRI), parks them by URL, preloads the font and the pack', async () => {
    localStorage.setItem('mw.boot.session', '1');
    page();
    run();
    expect(fetched.map((f) => f.url)).toEqual(MANIFEST.mail.map(abs));
    expect(fetched[0]!.init).toEqual({ credentials: 'same-origin', integrity: 'sha384-mail' });
    expect(fetched[2]!.init).toEqual({ credentials: 'same-origin' });
    expect(Object.keys(parked())).toEqual(MANIFEST.mail.map(abs));
    expect(await parked()[abs('./assets/mail-AAAA.js')]).toBe(true);
    expect(links()).toEqual([
      { as: 'font', href: './assets/jost-latin-wght-normal-FFFF.woff2', cors: '' },
      { as: 'fetch', href: './assets/catalog.en.app-HHHH.json', cors: '' },
    ]);
    expect(document.head.querySelector('link[as="font"]')!.getAttribute('type')).toBe('font/woff2');
  });

  it('without a session: the sign-in files instead; on a consent page: no chunk at all', () => {
    page();
    run();
    expect(fetched.map((f) => f.url)).toEqual(MANIFEST.login.map(abs));
    fetched = [];
    localStorage.setItem('mw.boot.session', '1');
    history.replaceState(null, '', '/oauth/authorize');
    page();
    run();
    expect(fetched).toEqual([]);
    expect(links().map((l) => l.as)).toEqual(['font', 'fetch']);
  });

  it('a failed download resolves false (the import retries it, chunkRetry.ts)', async () => {
    vi.stubGlobal('fetch', () => Promise.resolve(new Response(null, { status: 503 })));
    page();
    run();
    expect(await parked()[abs('./assets/login-DDDD.js')]).toBe(false);
    vi.stubGlobal('fetch', () => Promise.reject(new TypeError('offline')));
    page();
    run();
    expect(await parked()[abs('./assets/login-DDDD.js')]).toBe(false);
  });

  it('preloads the stored UI font override, and none for the system stack', () => {
    snapshot({ ...defaultAppearance(), chosen: true }, 'dark', { font: FONT_STACKS.inter! });
    page();
    run();
    expect(links().filter((l) => l.as === 'font').map((l) => l.href)).toEqual(['./assets/inter-latin-wght-normal-GGGG.woff2']);
    snapshot({ ...defaultAppearance(), chosen: true }, 'dark', { font: FONT_STACKS.system! });
    page();
    run();
    expect(links().filter((l) => l.as === 'font')).toEqual([]);
  });

  it('every UI font stack names a preloadable family (or the system font)', () => {
    for (const [name, stack] of Object.entries(FONT_STACKS)) {
      if (stack === null || name === 'system') continue;
      expect(UI_FONT_SLUGS, name).toContain(fontSlug(stack));
    }
    expect(fontSlug('system-ui, sans-serif')).toBe('system-ui');
    expect(UI_FONT_SLUGS).toContain(fontSlug('"Jost", system-ui'));
  });

  it.each([
    [['en-US']], [['de-AT', 'en']], [['pt']], [['pt-AO']], [['pt-BR']], [['pt-PT']], [['pt-MZ', 'de']], [['zh-TW']], [['zh-Hans-CN']], [['ja-JP']], [['es-419']],
    [['nb-NO', 'fr-CA']], [['xx']], [['uk-UA']], [['ar-EG']], [['nl-BE']], [['it-CH']], [['pl']], [['ru-RU']], [['sr-Latn', 'ru']],
  ])('picks the same locale as negotiateLocale for %j', (wanted: string[]) => {
    languages(wanted);
    page();
    run();
    expect(packLinks()).toEqual([`./assets/catalog.${negotiateLocale(wanted)}.app-HHHH.json`]);
  });

  it('a stored language choice wins over the browser; "browser" and unknown values do not', () => {
    languages(['de-DE']);
    for (const [stored, want] of [['fr', 'fr'], ['browser', 'de'], ['klingon', 'de']] as const) {
      localStorage.setItem('mw.locale', stored);
      page();
      run();
      expect(packLinks(), stored).toEqual([`./assets/catalog.${want}.app-HHHH.json`]);
    }
  });

  it('ignores a manifest it does not know, a path outside ./assets/, and the admin console', () => {
    page('<script type="application/json" id="mw-boot-manifest">{"v":2,"login":["./assets/x.js"]}</script>');
    run();
    page('<script type="application/json" id="mw-boot-manifest">{not json</script>');
    expect(run).not.toThrow();
    page({ ...MANIFEST, login: ['https://elsewhere.test/x.js', '/assets/y.js'], packs: {}, fonts: { jost: '//cdn.test/f.woff2' } });
    run();
    expect(fetched).toEqual([]);
    expect(links()).toEqual([]);
    page();
    const meta = document.createElement('meta');
    meta.name = 'mw-console';
    meta.content = 'admin';
    document.head.append(meta);
    run();
    expect(fetched).toEqual([]);
    expect(links()).toEqual([]);
  });
});

describe('the boot manifest (t52w, bootManifest.ts)', () => {
  const chunk = (fileName: string, name: string, imports: string[] = [], importedCss: string[] = [], isEntry = false): ManifestChunk => ({ type: 'chunk', fileName, name, imports, importedCss, isEntry });
  const bundle: ManifestChunk[] = [
    chunk('assets/index-1.js', 'index', ['assets/core-1.js', 'assets/vendor-core-1.js'], [], true),
    chunk('assets/core-1.js', 'core', ['assets/vendor-core-1.js'], ['assets/core-1.css']),
    chunk('assets/vendor-core-1.js', 'vendor-core'),
    chunk('assets/shared-1.js', 'shared', ['assets/core-1.js'], ['assets/shared-1.css']),
    chunk('assets/mail-1.js', 'mail', ['assets/shared-1.js', 'assets/core-1.js', 'assets/index-1.js'], ['assets/mail-1.css']),
    chunk('assets/login-1.js', 'login', ['assets/core-1.js'], ['assets/login-1.css']),
    { type: 'asset', fileName: 'assets/jost-latin-wght-normal-1.woff2' },
    { type: 'asset', fileName: 'assets/jost-cyrillic-wght-normal-1.woff2' },
    { type: 'asset', fileName: 'assets/lora-latin-wght-normal-1.woff2' },
    { type: 'asset', fileName: 'assets/catalog.pt-BR.app-1.json' },
    { type: 'asset', fileName: 'assets/catalog.pt-BR.admin-1.json' },
  ];

  it('lists each first screen beyond the entry, the UI latin fonts and the app packs', () => {
    expect(buildBootManifest(bundle)).toEqual({
      v: 1,
      mail: ['./assets/mail-1.js', './assets/shared-1.js', './assets/mail-1.css', './assets/shared-1.css'],
      login: ['./assets/login-1.js', './assets/login-1.css'],
      fonts: { jost: './assets/jost-latin-wght-normal-1.woff2' },
      packs: { 'pt-BR': './assets/catalog.pt-BR.app-1.json' },
      sri: {},
    });
  });

  it('fails the build when a first screen has no chunk', () => {
    expect(() => buildBootManifest(bundle.filter((c) => c.name !== 'login'))).toThrow(/no "login" chunk/);
  });

  it('the data block cannot be closed by its content, and reads back', () => {
    const manifest = { ...buildBootManifest(bundle), sri: { x: '</script><script>alert(1)</script>' } };
    const tag = manifestTag(manifest);
    expect(tag.match(/<\/script>/g)).toHaveLength(1);
    expect(readManifestTag(`<head>${tag}</head>`)!.manifest).toEqual(manifest);
  });
});

describe('boot.js starts GET /api/bootstrap for the app to adopt (t52w)', () => {
  let fetched: { url: string; init: RequestInit }[];
  const bootSrc = (src: string | null): void => {
    Object.defineProperty(document, 'currentScript', { configurable: true, get: () => (src === null ? null : { src }) });
  };
  beforeEach(() => {
    fetched = [];
    vi.stubGlobal('fetch', (url: string, init: RequestInit) => { fetched.push({ url, init }); return Promise.resolve(new Response('{}')); });
  });
  afterEach(() => {
    delete (document as { currentScript?: unknown }).currentScript;
    delete (window as { __mwBootstrap?: unknown }).__mwBootstrap;
    delete (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
  });

  it('under the deploy prefix the script is served from, parked as __mwBootstrap', () => {
    bootSrc('https://mail.example.test/mail/assets/boot-AbC.js');
    run();
    expect(fetched).toEqual([{ url: 'https://mail.example.test/mail/api/bootstrap', init: { credentials: 'same-origin', headers: { accept: 'application/json' } } }]);
    expect((window as { __mwBootstrap?: unknown }).__mwBootstrap).toBeInstanceOf(Promise);
  });

  it('a failed request is not an unhandled rejection', async () => {
    bootSrc('https://mail.example.test/assets/boot-AbC.js');
    vi.stubGlobal('fetch', () => Promise.reject(new TypeError('offline')));
    const unhandled = vi.fn();
    window.addEventListener('unhandledrejection', unhandled);
    run();
    await new Promise((resolve) => { setTimeout(resolve, 0); });
    window.removeEventListener('unhandledrejection', unhandled);
    expect(unhandled).not.toHaveBeenCalled();
    await expect((window as { __mwBootstrap?: Promise<Response> }).__mwBootstrap).rejects.toThrow('offline');
  });

  it('not in a native shell, not on the admin console', () => {
    bootSrc('https://mail.example.test/assets/boot-AbC.js');
    (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
    run();
    expect(fetched).toEqual([]);
    delete (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    const meta = document.createElement('meta');
    meta.name = 'mw-console';
    meta.content = 'admin';
    document.head.append(meta);
    run();
    expect(fetched).toEqual([]);
  });
});
