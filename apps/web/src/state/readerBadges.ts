// Which badges the reader's header row shows, and how: one visibility setting per badge
// (`always`, `only when notable`, `never`), plus a density and a label style shared by all.
// Each key is a global/domain/user setting, so the administrator can set the default or lock it.

import {settings} from '../api/settings.ts';

export const BADGE_IDS = ['auth', 'spam', 'sanitized', 'folder'] as const;
export type BadgeId = (typeof BADGE_IDS)[number];
export const BADGE_MODES = ['always', 'notable', 'never'] as const;
export type BadgeMode = (typeof BADGE_MODES)[number];
export const BADGE_DENSITIES = ['compact', 'comfortable'] as const;
export type BadgeDensity = (typeof BADGE_DENSITIES)[number];
export const BADGE_LABELS = ['text', 'icon'] as const;
export type BadgeLabels = (typeof BADGE_LABELS)[number];

export const BADGE_MODE_DEFAULTS: Record<BadgeId, BadgeMode> = {auth: 'always', spam: 'always', sanitized: 'notable', folder: 'always'};
export const BADGE_DENSITY_KEY = 'mail.badge.density';
export const BADGE_LABELS_KEY = 'mail.badge.labels';

export const badgeKey = (id: BadgeId): string => `mail.badge.${id}`;
export const isBadgeMode = (v: unknown): v is BadgeMode => typeof v === 'string' && (BADGE_MODES as readonly string[]).includes(v);
export const isBadgeDensity = (v: unknown): v is BadgeDensity => typeof v === 'string' && (BADGE_DENSITIES as readonly string[]).includes(v);
export const isBadgeLabels = (v: unknown): v is BadgeLabels => typeof v === 'string' && (BADGE_LABELS as readonly string[]).includes(v);

/** Reactive: the administrator's lock or default, else the user's choice. */
export function badgeMode(id: BadgeId): BadgeMode {
  const value = settings()[badgeKey(id)]?.value;
  return isBadgeMode(value) ? value : BADGE_MODE_DEFAULTS[id];
}
export function badgeDensity(): BadgeDensity {
  const value = settings()[BADGE_DENSITY_KEY]?.value;
  return isBadgeDensity(value) ? value : 'compact';
}
export function badgeLabels(): BadgeLabels {
  const value = settings()[BADGE_LABELS_KEY]?.value;
  return isBadgeLabels(value) ? value : 'text';
}
/** Whether an administrator holds this badge's setting. */
export function badgeLocked(id: BadgeId): boolean {
  return settings()[badgeKey(id)]?.locked === true;
}
/** Whether a badge shows: `notable` means something worth a look (a failure, a warning, a change, an unusual folder). */
export function badgeVisible(id: BadgeId, notable: boolean): boolean {
  const mode = badgeMode(id);
  return mode === 'always' || (mode === 'notable' && notable);
}
