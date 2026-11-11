import {formatDateWith} from '../../i18n/format.ts';
// The composer's bottom status bar: one fixed-height strip under the editor
// that holds every notice (draft save state, the identity hint, pre-send
// problems, upload and recall state), so nothing above the fields appears or
// disappears and the composer never jumps. This module is the pure model: the
// items, their order and how many fit; `ComposeStatusBar.tsx` renders it.

import type { IconName } from '../Icon.tsx';

export type StatusSeverity = 'danger' | 'warning' | 'busy' | 'neutral' | 'success';

export interface StatusItem {
  id: string;
  severity: StatusSeverity;
  icon: IconName;
  /** The short text shown in the bar. */
  label: string;
  /** The longer reason, shown in the themed popover (never the only place a state is stated). */
  detail?: string;
  /** Clicking the item (jump to the field, open the detail). */
  onClick?: () => void;
  /** A second, named action on the item ("Retry", "Change"). */
  actionLabel?: string;
  onAction?: () => void;
  /** Announce at once (role=alert) instead of politely. Errors only. */
  assertive?: boolean;
  /** A smaller, quieter item (outside trusted domains). */
  subdued?: boolean;
  testId?: string;
}

const RANK: Record<StatusSeverity, number> = { danger: 0, warning: 1, busy: 2, neutral: 3, success: 4 };

/** Most urgent first; items of one severity keep the order they were given in. */
export function orderItems(items: readonly StatusItem[]): StatusItem[] {
  // The draft save state is always last, so it sits rightmost on the bar.
  const rank = (item: StatusItem): number => (item.id === 'draft' ? 99 : RANK[item.severity]);
  return items.map((item, index) => ({ item, index })).sort((a, b) => rank(a.item) - rank(b.item) || a.index - b.index).map(({ item }) => item);
}

/** How many items fit inline for a bar `width` px wide; the rest go behind the count chip. */
export function inlineCapacity(width: number, itemWidth = 230, chipWidth = 56): number {
  if (!Number.isFinite(width) || width <= 0) return 6;
  return Math.max(1, Math.floor((width - chipWidth) / itemWidth));
}

export function splitItems(items: readonly StatusItem[], capacity: number): { inline: StatusItem[]; overflow: StatusItem[] } {
  const ordered = orderItems(items);
  if (ordered.length <= capacity) return { inline: ordered, overflow: [] };
  return { inline: ordered.slice(0, capacity), overflow: ordered.slice(capacity) };
}

/** The clock time shown next to "Saved", in the user's locale. */
export function savedTime(at: Date, locale?: string): string {
  try {
    return formatDateWith(at,{hour:'2-digit',minute:'2-digit'},locale);
  } catch {
    return at.toISOString().slice(11, 16);
  }
}
