/** Case- and diacritic-insensitive text matching for the settings search. */
export const normalise = (s: string): string => s.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();
export const tokens = (query: string): string[] => normalise(query).split(/\s+/).filter(Boolean);
/** Every query word must appear somewhere in `text`. */
export function cardMatches(text: string, words: readonly string[]): boolean {
  if (!words.length) return true;
  const hay = normalise(text);
  return words.every(w => hay.includes(w));
}
