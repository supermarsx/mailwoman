import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { manifestTag, type BootManifest } from './bootManifest.ts';
import { CONTROLLER_WAIT_MS, FILL_GAP_MS, fillList, idleFill, type FillHost } from './idleFill.ts';

const MANIFEST: BootManifest = {
  v: 1, mail: [], login: [], fonts: {}, sri: {},
  packs: { en: './assets/catalog.en.app-A.json', de: './assets/catalog.de.app-B.json' },
  idle: ['./assets/settings-C.js', './assets/pim-D.js', './assets/settings-E.css', 'https://elsewhere.test/x.js'],
};
const abs = (href: string): string => new URL(href, document.baseURI).href;

function install(manifest: BootManifest = MANIFEST): void {
  document.getElementById('mw-boot-manifest')?.remove();
  const holder = document.createElement('div');
  holder.innerHTML = manifestTag(manifest);
  document.head.append(holder.firstElementChild!);
}

function host(extra: Partial<FillHost> = {}): FillHost & { log: string[] } {
  const log: string[] = [];
  return {
    log,
    document,
    serviceWorker: undefined,
    native: false,
    sleep: async (ms) => { log.push(`wait ${ms}`); },
    fetchFile: async (url) => { log.push(url); return true; },
    ...extra,
  };
}

beforeEach(() => { install(); document.documentElement.lang = 'de'; });
afterEach(() => { document.getElementById('mw-boot-manifest')?.remove(); document.documentElement.lang = 'en'; });

describe('idle fill (t52w)', () => {
  it('the active locale\'s app pack first, then the Settings/PIM files; only ./assets/ paths', () => {
    expect(fillList(document)).toEqual(['./assets/catalog.de.app-B.json', './assets/settings-C.js', './assets/pim-D.js', './assets/settings-E.css'].map(abs));
    document.documentElement.lang = 'tlh';
    expect(fillList(document)).toEqual(['./assets/settings-C.js', './assets/pim-D.js', './assets/settings-E.css'].map(abs));
  });

  it('one file at a time with a pause between files', async () => {
    const h = host();
    await idleFill(h);
    expect(h.log).toEqual([abs('./assets/catalog.de.app-B.json'), `wait ${FILL_GAP_MS}`, abs('./assets/settings-C.js'), `wait ${FILL_GAP_MS}`, abs('./assets/pim-D.js'), `wait ${FILL_GAP_MS}`, abs('./assets/settings-E.css')]);
  });

  it('waits for the service worker to take control, at most CONTROLLER_WAIT_MS', async () => {
    let listener: (() => void) | undefined;
    const container = { controller: null as unknown, addEventListener: (_: 'controllerchange', l: () => void) => { listener = l; }, removeEventListener: () => { listener = undefined; } };
    let woke: () => void = () => undefined;
    const h = host({ serviceWorker: container, sleep: (ms) => (ms === CONTROLLER_WAIT_MS ? new Promise<void>((resolve) => { woke = resolve; }) : Promise.resolve()) });
    const done = idleFill(h);
    await Promise.resolve();
    expect(h.log).toEqual([]);
    listener!();
    await done;
    expect(h.log[0]).toBe(abs('./assets/catalog.de.app-B.json'));
    // Without a controller ever: the timeout lets it go ahead.
    const h2 = host({ serviceWorker: { ...container, controller: null }, sleep: (ms) => (ms === CONTROLLER_WAIT_MS ? new Promise<void>((resolve) => { woke = resolve; }) : Promise.resolve()) });
    const done2 = idleFill(h2);
    await Promise.resolve();
    expect(h2.log).toEqual([]);
    woke();
    await done2;
    expect(h2.log).toHaveLength(4);
  });

  it('nothing in a native shell or without a manifest', async () => {
    const h = host({ native: true });
    await idleFill(h);
    expect(h.log).toEqual([]);
    document.getElementById('mw-boot-manifest')?.remove();
    const h2 = host();
    await idleFill(h2);
    expect(h2.log).toEqual([]);
  });
});
