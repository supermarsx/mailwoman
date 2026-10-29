import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { ICON_NAMES } from '../components/Icon.tsx';
import { AVATAR_COLORS } from '../components/senderIcons/palette.ts';
import { contrastRatio } from '../theme/contrast.ts';
import {
  AVATAR_CONTRAST_MIN, AVATAR_ICONS, autoForeground, avatarIconLabel, avatarLook, cleanAvatarText, cleanHex, isDefaultAvatar, validAvatar,
} from './mailboxAvatar.ts';
import { layoutChange } from './mailboxLayout.ts';
import type { LinkedMailboxList } from './linkedAccounts.ts';

describe('mailbox avatars (t86)', () => {
  it('offers 26 icons, each drawn by the icon module, with its own name', () => {
    expect(AVATAR_ICONS).toHaveLength(26);
    expect(new Set(AVATAR_ICONS.map((icon) => icon.id)).size).toBe(26);
    expect(new Set(AVATAR_ICONS.map((icon) => icon.glyph)).size).toBe(26);
    for (const icon of AVATAR_ICONS) expect(ICON_NAMES).toContain(icon.glyph);
    expect(avatarIconLabel('heartHome')).toBe('linked-accounts-avatar-icon-heart-home');
    expect(avatarIconLabel('eyeOff')).toBe('linked-accounts-avatar-icon-eye-off');
  });

  it('the icon allowlist is the same list the server enforces', () => {
    const routes = readFileSync(join(process.cwd(), '../../crates/mw-server/src/linked_accounts/routes.rs'), 'utf8');
    const block = routes.slice(routes.indexOf('pub(crate) const AVATAR_ICONS'), routes.indexOf('];', routes.indexOf('pub(crate) const AVATAR_ICONS')));
    const server = [...block.matchAll(/"([A-Za-z]+)"/g)].map((m) => m[1]);
    expect(server).toEqual(AVATAR_ICONS.map((icon) => icon.id));
  });

  it('every icon has an en and pt-PT name (tooltip and accessible name)', () => {
    for (const locale of ['en', 'pt-PT']) {
      const ftl = readFileSync(join(process.cwd(), `locales/${locale}/linked-accounts.ftl`), 'utf8');
      for (const icon of AVATAR_ICONS) expect(ftl, `${locale} ${icon.id}`).toMatch(new RegExp(`^${avatarIconLabel(icon.id)} = \\S`, 'm'));
    }
  });

  it('Auto picks black or white and always meets AA against the background', () => {
    const samples = ['#000000', '#ffffff', '#777777', '#767676', '#808080', '#1d4ed8', '#fde68a', '#ff0000', '#00ff00', ...AVATAR_COLORS];
    for (const bg of samples) {
      const fg = autoForeground(bg);
      expect(['#000000', '#ffffff']).toContain(fg);
      expect(contrastRatio(fg, bg), bg).toBeGreaterThanOrEqual(AVATAR_CONTRAST_MIN);
    }
    expect(autoForeground('#fde68a')).toBe('#000000');
    expect(autoForeground('#1d4ed8')).toBe('#ffffff');
  });

  it('a letter is one or two visible characters', () => {
    expect(cleanAvatarText(' W ')).toBe('W');
    expect(cleanAvatarText('Ab')).toBe('Ab');
    expect(cleanAvatarText('日本')).toBe('日本');
    expect(cleanAvatarText('\u{1F600}')).toBe('\u{1F600}');
    expect(cleanAvatarText('')).toBe('');
    for (const bad of ['ABC', 'A B', 'A\u202E', '\u200BA', 'A\u00AD', 'A'+String.fromCharCode(10)+'B']) expect(cleanAvatarText(bad), JSON.stringify(bad)).toBeNull();
  });

  it('only valid fields from the server survive', () => {
    expect(cleanHex('#ABCDEF')).toBe('#abcdef');
    expect(cleanHex('#abc')).toBeUndefined();
    expect(validAvatar({ kind: 'icon', icon: 'house', fg: '#FFFFFF', bg: 'red' })).toEqual({ kind: 'icon', icon: 'house', fg: '#ffffff' });
    expect(validAvatar({ kind: 'icon', icon: 'rocket' })).toBeUndefined();
    expect(validAvatar({ kind: 'letter', text: 'XYZ' })).toBeUndefined();
    expect(validAvatar({ kind: 'letter', text: 'Ç', bg: '#112233' })).toEqual({ kind: 'letter', text: 'Ç', bg: '#112233' });
    expect(validAvatar('house')).toBeUndefined();
    expect(validAvatar(null)).toBeUndefined();
    expect(isDefaultAvatar({ kind: 'letter' })).toBe(true);
    expect(isDefaultAvatar({ kind: 'letter', bg: '#000000' })).toBe(false);
  });

  it('no saved avatar keeps the generated look; a saved one uses its colours', () => {
    const plain = avatarLook(undefined, 'Work', 'work@example.org');
    expect(plain).toMatchObject({ kind: 'letter', text: 'W', custom: false, fg: '#ffffff' });
    expect(plain.palette).toBeTypeOf('number');
    const icon = avatarLook({ kind: 'icon', icon: 'briefcase', bg: '#fde68a' }, 'Work', 'work@example.org');
    expect(icon).toMatchObject({ kind: 'icon', glyph: 'briefcase', bg: '#fde68a', fg: '#000000', autoFg: true, custom: true });
    expect(icon.palette).toBeUndefined();
    const letter = avatarLook({ kind: 'letter', text: 'Ab', fg: '#777777', bg: '#808080' }, 'Work', 'work@example.org');
    expect(letter).toMatchObject({ text: 'Ab', autoFg: false });
    expect(letter.ratio).toBeLessThan(AVATAR_CONTRAST_MIN);
  });

  it('a layout change carries the avatars and drops a reset one', () => {
    const list = { primary: { accountId: '1', address: 'a@example.org', storageAuthority: 'x' }, accounts: [], policy: { enabled: true, maxAccounts: 4, supportedAuthentication: [] },
      layout: { revision: 'r', order: ['main'], nicknames: {}, avatars: { main: { kind: 'icon', icon: 'lock' } } } } as LinkedMailboxList;
    expect(layoutChange(list, { order: ['main'] }).avatars).toEqual({ main: { kind: 'icon', icon: 'lock' } });
    expect(layoutChange(list, { avatar: { key: 'main', value: { kind: 'letter', text: 'Z' } } }).avatars).toEqual({ main: { kind: 'letter', text: 'Z' } });
    expect(layoutChange(list, { avatar: { key: 'main', value: undefined } }).avatars).toEqual({});
  });
});
