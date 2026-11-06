// Sorting a selection (FORMAT): the order, type and options the Sort control
// offers, applied to a list of lines, paragraphs, list items or table rows.
// Editor-independent: the two editors pass in the text of each item and get back
// the new order.

export type SortType = 'text' | 'number' | 'date';
export type SortMode = 'sort' | 'reverse' | 'shuffle' | 'unique';

export interface SortOptions {
  mode: SortMode;
  order: 'asc' | 'desc';
  type: SortType;
  caseSensitive: boolean;
  /** Natural order: `item2` before `item10`. */
  natural: boolean;
  ignoreArticles: boolean;
  /** Drop an item whose sort key repeats an earlier one. */
  removeDuplicates: boolean;
  /** Tables: the first row stays where it is. */
  header: boolean;
  /** Tables: the column to sort by (0-based); null = the caret's column. */
  column: number | null;
}

export const DEFAULT_SORT: SortOptions = { mode: 'sort', order: 'asc', type: 'text', caseSensitive: false, natural: true, ignoreArticles: false, removeDuplicates: false, header: true, column: null };

const ARTICLES = /^(?:the|an?|o|os|as|um|uma|le|la|les|un|une|el|los|las|der|die|das|ein|eine|il|lo|gli|de|het)\s+/i;
const NUMBER = /-?\d+(?:[.,]\d+)?/;

/** The text an item sorts by. */
export function sortKeyText(text: string, options: Pick<SortOptions, 'caseSensitive' | 'ignoreArticles'>): string {
  let key = text.trim().replace(/\s+/g, ' ');
  if (options.ignoreArticles) key = key.replace(ARTICLES, '');
  return options.caseSensitive ? key : key.toLocaleLowerCase();
}

function numberOf(text: string): number | null {
  const match = NUMBER.exec(text);
  if (!match) return null;
  const value = Number(match[0].replace(',', '.'));
  return Number.isFinite(value) ? value : null;
}

function dateOf(text: string): number | null {
  const trimmed = text.trim();
  const day = /^(\d{1,2})[/.](\d{1,2})[/.](\d{4})$/.exec(trimmed);
  const value = day ? Date.UTC(Number(day[3]), Number(day[2]) - 1, Number(day[1])) : Date.parse(trimmed);
  return Number.isFinite(value) ? value : null;
}

export interface SortResult<T> { items: T[]; removed: number; changed: boolean }

/**
 * Order `items`. Items without a number or date (for those types) go last, in their
 * original order whichever way the sort runs. The sort is stable.
 */
export function sortItems<T>(items: readonly T[], textOf: (item: T) => string, options: SortOptions, random: () => number = Math.random): SortResult<T> {
  let list = items.map((item, index) => ({ item, index, text: textOf(item) }));
  const before = list.map((entry) => entry.item);
  let removed = 0;
  if (options.removeDuplicates || options.mode === 'unique') {
    const seen = new Set<string>();
    list = list.filter((entry) => {
      const key = sortKeyText(entry.text, options);
      if (seen.has(key)) { removed += 1; return false; }
      seen.add(key);
      return true;
    });
  }
  if (options.mode === 'reverse') list = [...list].reverse();
  else if (options.mode === 'shuffle') {
    list = [...list];
    for (let i = list.length - 1; i > 0; i -= 1) { const j = Math.floor(random() * (i + 1)); [list[i], list[j]] = [list[j]!, list[i]!]; }
  } else if (options.mode === 'sort') {
    const direction = options.order === 'desc' ? -1 : 1;
    const collator = new Intl.Collator(undefined, { numeric: options.natural, sensitivity: options.caseSensitive ? 'variant' : 'base' });
    if (options.type === 'text') {
      list = [...list].sort((a, b) => direction * collator.compare(sortKeyText(a.text, options), sortKeyText(b.text, options)) || a.index - b.index);
    } else {
      const read = options.type === 'number' ? numberOf : dateOf;
      const keyed = list.map((entry) => ({ ...entry, key: read(entry.text) }));
      const present = keyed.filter((entry) => entry.key !== null).sort((a, b) => direction * (a.key! - b.key!) || a.index - b.index);
      list = [...present, ...keyed.filter((entry) => entry.key === null)];
    }
  }
  const out = list.map((entry) => entry.item);
  return { items: out, removed, changed: out.length !== before.length || out.some((item, index) => item !== before[index]) };
}
