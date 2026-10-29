// t86: the owner's avatar for each of their mailboxes (the sign-in account and
// every link): a letter (one or two characters) or one of a fixed set of icons,
// with a foreground (letter or icon) colour and a background colour. Kept on
// the server per owner next to the nicknames and order (`layout.avatars`, key =
// the layout key). Display only, like a nickname.
//
// No colour stored = the default: the background is the generated avatar colour
// (palette index from the address, as before) and the foreground is chosen
// automatically for WCAG AA contrast against the background.
import type { IconName } from '../components/Icon.tsx';
import { avatarPresentation } from '../components/PersonAvatar.tsx';
import { AVATAR_COLORS } from '../components/senderIcons/palette.ts';
import { contrastRatio } from '../theme/contrast.ts';

/** One saved avatar. `text` is only for `letter` (absent = the default initial); `icon` only for `icon`. */
export interface MailboxAvatarSpec {
  kind: 'letter' | 'icon';
  text?: string;
  icon?: AvatarIconId;
  /** `#rrggbb`; absent = Auto (black or white, whichever passes AA against the background). */
  fg?: string;
  /** `#rrggbb`; absent = the generated colour for the address. */
  bg?: string;
}

/**
 * The icons an avatar may use, in the order the editor shows them, grouped by
 * purpose. The id is what is stored (the server keeps the same allowlist in
 * crates/mw-server/src/linked_accounts/routes.rs, AVATAR_ICONS); `glyph` is the
 * drawing in components/Icon.tsx; `group` orders the grid.
 */
export const AVATAR_ICONS = [
  { id: 'briefcase', glyph: 'briefcase', group: 'work' },
  { id: 'office', glyph: 'office', group: 'work' },
  { id: 'laptop', glyph: 'laptop', group: 'work' },
  { id: 'team', glyph: 'team', group: 'work' },
  { id: 'store', glyph: 'store', group: 'business' },
  { id: 'chart', glyph: 'chart', group: 'business' },
  { id: 'house', glyph: 'home', group: 'home' },
  { id: 'heartHome', glyph: 'heartHome', group: 'home' },
  { id: 'family', glyph: 'family', group: 'home' },
  { id: 'person', glyph: 'person', group: 'personal' },
  { id: 'smile', glyph: 'smile', group: 'personal' },
  { id: 'star', glyph: 'star', group: 'personal' },
  { id: 'lock', glyph: 'lock', group: 'private' },
  { id: 'shield', glyph: 'shield', group: 'private' },
  { id: 'eyeOff', glyph: 'eyeOff', group: 'private' },
  { id: 'key', glyph: 'key', group: 'admin' },
  { id: 'gear', glyph: 'gear', group: 'admin' },
  { id: 'crown', glyph: 'crown', group: 'admin' },
  { id: 'wrench', glyph: 'wrench', group: 'utility' },
  { id: 'bolt', glyph: 'bolt', group: 'utility' },
  { id: 'inbox', glyph: 'inbox', group: 'utility' },
  { id: 'receipt', glyph: 'receipt', group: 'utility' },
  { id: 'bell', glyph: 'bell', group: 'utility' },
  { id: 'cart', glyph: 'cart', group: 'other' },
  { id: 'plane', glyph: 'travel', group: 'other' },
  { id: 'school', glyph: 'education', group: 'other' },
] as const satisfies readonly { id: string; glyph: IconName; group: string }[];

export type AvatarIconId = (typeof AVATAR_ICONS)[number]['id'];
const ICON_IDS: ReadonlySet<string> = new Set(AVATAR_ICONS.map((icon) => icon.id));

/** The catalogue id of an icon's name (tooltip and accessible name). */
export const avatarIconLabel = (id: AvatarIconId): string => `linked-accounts-avatar-icon-${id.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`)}`;
export const avatarGlyph = (id: AvatarIconId): IconName => AVATAR_ICONS.find((icon) => icon.id === id)!.glyph;
export const isAvatarIcon = (value: unknown): value is AvatarIconId => typeof value === 'string' && ICON_IDS.has(value);

export const AVATAR_TEXT_MAX = 2;
const HEX = /^#[0-9a-f]{6}$/;
// Refused in the letter, as in a nickname (the server refuses the same set):
// controls, format characters (zero-width, bidirectional), and whitespace.
const REFUSED_TEXT = /[\p{Cc}\p{Cf}\p{Z}\s\u061C\u200B-\u200F\u202A-\u202E\u2060-\u206F\uFEFF\uFFF9-\uFFFB]/u;

/** A letter as typed: trimmed; `''` = the default initial; `null` = not acceptable. */
export function cleanAvatarText(raw: string): string | null {
  const value = raw.trim();
  if (Array.from(value).length > AVATAR_TEXT_MAX || REFUSED_TEXT.test(value)) return null;
  return value;
}

export function cleanHex(raw: string | null | undefined): string | undefined {
  const value = raw?.trim().toLowerCase();
  return value && HEX.test(value) ? value : undefined;
}

/** A spec from the server (or anything else): only valid fields survive; `undefined` when nothing valid. */
export function validAvatar(value: unknown): MailboxAvatarSpec | undefined {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return undefined;
  const raw = value as Record<string, unknown>;
  const fg = cleanHex(typeof raw['fg'] === 'string' ? raw['fg'] : undefined);
  const bg = cleanHex(typeof raw['bg'] === 'string' ? raw['bg'] : undefined);
  const colours = { ...(fg ? { fg } : {}), ...(bg ? { bg } : {}) };
  if (raw['kind'] === 'icon') return isAvatarIcon(raw['icon']) ? { kind: 'icon', icon: raw['icon'], ...colours } : undefined;
  if (raw['kind'] !== 'letter') return undefined;
  const text = typeof raw['text'] === 'string' ? cleanAvatarText(raw['text']) : '';
  if (text === null) return undefined;
  return { kind: 'letter', ...(text ? { text } : {}), ...colours };
}

/** True when the spec is the default look (nothing worth storing). */
export const isDefaultAvatar = (spec: MailboxAvatarSpec | undefined): boolean =>
  !spec || (spec.kind === 'letter' && !spec.text && !spec.fg && !spec.bg);

const BLACK = '#000000';
const WHITE = '#ffffff';
/** AA for normal text: the letter is small, so it is judged as text (4.5:1). */
export const AVATAR_CONTRAST_MIN = 4.5;

/** Black or white, whichever contrasts more with `bg` (either one always reaches at least 4.58:1). */
export function autoForeground(bg: string): string {
  return contrastRatio(WHITE, bg) >= contrastRatio(BLACK, bg) ? WHITE : BLACK;
}

export interface AvatarLook {
  kind: 'letter' | 'icon';
  /** The letter shown (`kind: 'letter'`). */
  text: string;
  glyph?: IconName;
  icon?: AvatarIconId;
  /** The generated palette index; `undefined` when the background is custom. */
  palette?: number;
  bg: string;
  fg: string;
  /** The foreground was chosen automatically. */
  autoFg: boolean;
  /** A saved avatar: its colours are applied inline (else the generated palette class). */
  custom: boolean;
  /** Contrast of foreground against background. */
  ratio: number;
}

/**
 * What an avatar shows. `label` and `address` give the default letter (the
 * initials of the nickname, else of the address) and the generated colour.
 */
export function avatarLook(spec: MailboxAvatarSpec | undefined, label: string, address: string): AvatarLook {
  const generated = avatarPresentation(label === address ? '' : label, address);
  // No saved avatar: exactly the look it always had (generated colour, white initials).
  if (!spec) {
    const bg = AVATAR_COLORS[generated.palette]!;
    return { kind: 'letter', text: generated.initials, palette: generated.palette, bg, fg: WHITE, autoFg: true, custom: false, ratio: ratio(WHITE, bg) };
  }
  const bg = spec.bg ?? AVATAR_COLORS[generated.palette]!;
  const fg = spec.fg ?? autoForeground(bg);
  const base = { bg, fg, autoFg: !spec.fg, custom: true, ratio: ratio(fg, bg), ...(spec.bg ? {} : { palette: generated.palette }) };
  if (spec.kind === 'icon' && spec.icon) return { kind: 'icon', text: '', glyph: avatarGlyph(spec.icon), icon: spec.icon, ...base };
  return { kind: 'letter', text: spec.text || generated.initials, ...base };
}

const ratio = (fg: string, bg: string): number => Math.round(contrastRatio(fg, bg) * 100) / 100;

/** The default letter (the editor's placeholder): the initials the avatar shows without a saved letter. */
export const defaultAvatarText = (label: string, address: string): string => avatarPresentation(label === address ? '' : label, address).initials;
