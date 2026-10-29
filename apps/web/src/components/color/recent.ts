// Recently used colours of the colour picker (t81), per signed-in owner and per
// browser. There is no server preference for picker history, and a list of
// colours is a convenience: storage that is off or full just means no recents.

import { normalizeHex } from './colorMath.ts';

const MAX = 8;
const key = (owner: string): string => `mw.color-recent.v1:${owner || 'local'}`;

export function recentColors(owner: string): string[] {
  try {
    const raw: unknown = JSON.parse(localStorage.getItem(key(owner)) ?? '[]');
    if (!Array.isArray(raw)) return [];
    const out: string[] = [];
    for (const item of raw) {
      const hex = typeof item === 'string' ? normalizeHex(item, true) : null;
      if (hex && !out.includes(hex)) out.push(hex);
    }
    return out.slice(0, MAX);
  } catch {
    return [];
  }
}

/** Put `color` first in the owner's list; returns the new list. */
export function rememberColor(owner: string, color: string): string[] {
  const hex = normalizeHex(color, true);
  if (!hex) return recentColors(owner);
  const next = [hex, ...recentColors(owner).filter((item) => item !== hex)].slice(0, MAX);
  try {
    localStorage.setItem(key(owner), JSON.stringify(next));
  } catch {
    /* Storage off: the list lives only for this session's open picker. */
  }
  return next;
}
