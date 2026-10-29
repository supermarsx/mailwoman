// t95: every admin breadcrumb level resolves to a friendly name.
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { humanize, isOpaqueId } from '../../../components/breadcrumbs/humanize.ts';
import {
  crumbCatalogsLoaded, CRUMB_WORDS, ENTITY_KINDS, entityNoun, restCrumbLabels, sectionCrumbLabel, setEntityFetchers,
  settingCrumbLabel, specFor, wordLabel, type EntityKind,
} from './crumbLabels.ts';
import { KNOWN_KEYS } from '../Settings/keys.ts';
import { settingLabel } from '../Settings/labels.ts';
import { SECTIONS } from './sections.ts';

const DOTTED = /^[a-zA-Z0-9]+(?:\.[a-zA-Z0-9*-]+)+$/;
const MESSAGE_ID = /^[a-z][a-z0-9]*(?:-[a-z0-9]+)+$/;
const settle = async (): Promise<void> => { for (let i = 0; i < 5; i++) await Promise.resolve(); await new Promise((r) => setTimeout(r, 0)); };

beforeAll(async () => {
  await crumbCatalogsLoaded();
});
afterEach(() => setEntityFetchers());

describe('humanize', () => {
  it('turns keys and ids into sentence-case words', () => {
    expect(humanize('mail.focusedInbox')).toBe('Mail focused inbox');
    expect(humanize('alias-new')).toBe('Alias new');
    expect(humanize('+register')).toBe('Register');
    expect(humanize('max_bytes')).toBe('Max bytes');
    expect(humanize('security.signin.geoScope')).toBe('Security signin geo scope');
  });
  it('recognises opaque ids', () => {
    expect(isOpaqueId('3f2b6c1e-9a7d-4d2e-8f00-1234567890ab')).toBe(true);
    expect(isOpaqueId('a1b2c3d4e5f60718')).toBe(true);
    expect(isOpaqueId('42')).toBe(true);
    expect(isOpaqueId('corp-oidc')).toBe(false);
    expect(isOpaqueId('example.org')).toBe(false);
  });
});

describe('page words', () => {
  it('every word has its own label, not the word or a message id', () => {
    for (const word of CRUMB_WORDS) {
      const label = wordLabel(word);
      expect(label, word).not.toBeNull();
      expect(label, word).not.toBe(word);
      expect(label!, word).not.toMatch(MESSAGE_ID);
    }
  });
  it('marker spellings read as their word', () => {
    expect(specFor('users', ['+new'], 0)).toEqual({ kind: 'word', word: 'new' });
    expect(specFor('plugins', ['+register'], 0)).toEqual({ kind: 'word', word: 'register' });
  });
});

describe('sections', () => {
  it('every section label is a name, not a message id', () => {
    for (const s of SECTIONS) {
      const label = sectionCrumbLabel(s.id, s.label);
      expect(label, s.id).not.toBeNull();
      expect(label!, s.id).not.toMatch(MESSAGE_ID);
    }
  });
  it('a missing section label falls back to words from the id, logged at debug level', () => {
    const debug = vi.spyOn(console, 'debug').mockImplementation(() => {});
    expect(sectionCrumbLabel('t95-missing', () => 'admin-t95-missing-title')).toBe('T95 missing');
    expect(debug).toHaveBeenCalledWith(expect.stringContaining('t95-missing'));
    debug.mockRestore();
  });
});

describe('settings keys', () => {
  it('walks every registry key: the label is never the raw dotted key', () => {
    const raw: string[] = [];
    for (const key of KNOWN_KEYS) {
      const label = settingCrumbLabel(key, settingLabel);
      if (label === null || label === key || DOTTED.test(label) || MESSAGE_ID.test(label)) raw.push(`${key} -> ${label}`);
    }
    expect(raw).toEqual([]);
  });
  it('the settings and privacy routes resolve keys to their registry labels', () => {
    expect(restCrumbLabels('settings', ['mail.focusedInbox'], settingLabel)).toEqual([settingLabel('mail.focusedInbox')]);
    expect(settingLabel('mail.focusedInbox')).not.toBe('mail.focusedInbox');
    const [policy, key] = restCrumbLabels('privacy', ['policy', 'privacy.trackers'], settingLabel);
    expect(policy).toBe(wordLabel('policy'));
    expect(key).toBe(settingLabel('privacy.trackers'));
  });
  it('a key with no registry label falls back to words, never the dotted key', () => {
    const debug = vi.spyOn(console, 'debug').mockImplementation(() => {});
    expect(settingCrumbLabel('t95.someNewKey', settingLabel)).toBe('T95 some new key');
    expect(debug).toHaveBeenCalled();
    debug.mockRestore();
  });
});

describe('entities', () => {
  const ROUTES: readonly [string, string[], number, EntityKind][] = [
    ['users', ['3f2b6c1e-9a7d-4d2e-8f00-1234567890ab'], 0, 'user'],
    ['admins', ['a1'], 0, 'admin'],
    ['roles', ['r1', 'edit'], 0, 'role'],
    ['sso', ['corp-oidc', 'edit'], 0, 'sso'],
    ['identity-provider', ['clients', 'c1', 'edit'], 1, 'idp-client'],
    ['plugins', ['acme-spam'], 0, 'plugin'],
    ['ui-plugins', ['p1'], 0, 'ui-plugin'],
    ['features', ['restricted-staff'], 0, 'profile'],
    ['nextcloud', ['n1'], 0, 'nextcloud'],
    ['integrations', ['hosting', 'h1', 'edit'], 1, 'hosting'],
    ['integrations', ['ssh', 's1', 'edit'], 1, 'ssh'],
    ['egress', ['e1'], 0, 'egress-proxy'],
    ['egress', ['destinations', 'd1'], 1, 'egress-destination'],
    ['domains', ['example.org', 'backends', 'b1'], 2, 'backend'],
  ];

  it('each kind is recognised in its route', () => {
    for (const [section, rest, i, kind] of ROUTES) expect(specFor(section, rest, i), `${section}/${rest.join('/')}`).toMatchObject({ kind: 'entity', entity: kind, id: rest[i] });
    expect(new Set(ROUTES.map((r) => r[3]))).toEqual(new Set(ENTITY_KINDS));
  });

  it('shows a placeholder (null) while the name loads, then the display name; never the id', async () => {
    const fetchers = Object.fromEntries(ENTITY_KINDS.map((k) => [k, vi.fn(async (id: string) => `Name of ${k} ${id.length}`)]));
    setEntityFetchers(fetchers);
    for (const [section, rest, i, kind] of ROUTES) {
      const first = restCrumbLabels(section, rest, settingLabel)[i];
      expect(first, kind).toBeNull();
    }
    await settle();
    for (const [section, rest, i, kind] of ROUTES) {
      const label = restCrumbLabels(section, rest, settingLabel)[i];
      expect(label, kind).toBe(`Name of ${kind} ${rest[i]!.length}`);
      expect(label).not.toBe(rest[i]);
    }
    expect(fetchers['backend']).toHaveBeenCalledWith('b1', 'example.org');
  });

  it('an item with no name shows its kind for an opaque id, words otherwise', async () => {
    setEntityFetchers({ user: async () => undefined, sso: async () => { throw new Error('gone'); } });
    restCrumbLabels('users', ['3f2b6c1e-9a7d-4d2e-8f00-1234567890ab'], settingLabel);
    restCrumbLabels('sso', ['corp-oidc'], settingLabel);
    await settle();
    expect(restCrumbLabels('users', ['3f2b6c1e-9a7d-4d2e-8f00-1234567890ab'], settingLabel)).toEqual([entityNoun('user')]);
    expect(entityNoun('user')).toBe('User');
    expect(restCrumbLabels('sso', ['corp-oidc'], settingLabel)).toEqual(['Corp oidc']);
  });

  it('every kind has a noun', () => {
    for (const kind of ENTITY_KINDS) expect(entityNoun(kind)!, kind).not.toMatch(MESSAGE_ID);
  });
});

describe('route grammar', () => {
  it('domains, hosting names and tabs', () => {
    expect(restCrumbLabels('domains', ['example.org', 'connections'], settingLabel)).toEqual(['example.org', wordLabel('connections')]);
    expect(restCrumbLabels('domains', ['new', 'guided'], settingLabel)).toEqual([wordLabel('new'), wordLabel('guided')]);
    expect(restCrumbLabels('security', ['bans', 'new'], settingLabel)).toEqual([wordLabel('bans'), wordLabel('new')]);
    expect(specFor('integrations', ['hosting', 'h1', 'domains', 'example.org', 'mailboxes', 'alice'], 3)).toEqual({ kind: 'name', text: 'example.org' });
    expect(specFor('integrations', ['hosting', 'h1', 'domains', 'example.org', 'mailboxes', 'alice'], 4)).toEqual({ kind: 'word', word: 'mailboxes' });
  });
  it('an unknown segment is humanized, an unknown opaque one is "Item"', () => {
    const debug = vi.spyOn(console, 'debug').mockImplementation(() => {});
    expect(restCrumbLabels('maintenance', ['weird-tab'], settingLabel)).toEqual(['Weird tab']);
    expect(restCrumbLabels('maintenance', ['0123456789abcdef'], settingLabel)).toEqual(['Item']);
    debug.mockRestore();
  });
});
