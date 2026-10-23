// Pseudo-locale for layout testing (t32-admin §2.9).
//
// A build made with `MW_I18N_PSEUDO=1` (vite.config.ts turns it into the
// compile-time constant `__MW_I18N_PSEUDO__`) passes every `t()` result through
// `pseudoLocalize`: letters become accented look-alikes, the text grows by 40 %
// and is wrapped in brackets, e.g. "Save" -> "[Šáṽé ~~]". On that build:
//   - text that is not accented came from somewhere other than t() (hard-coded),
//   - a missing "]" means the text was cut off,
//   - a layout that breaks shows where a longer translation will not fit.
// In normal builds the constant is `false`, so the branch in registry.ts and
// this module are removed from the bundle.

const ACCENTED: Readonly<Record<string, string>> = {
  a: 'á', b: 'ƀ', c: 'ç', d: 'ð', e: 'é', f: 'ƒ', g: 'ĝ', h: 'ĥ', i: 'í', j: 'ĵ', k: 'ķ', l: 'ĺ', m: 'ɱ',
  n: 'ñ', o: 'ó', p: 'þ', q: 'ǫ', r: 'ŕ', s: 'š', t: 'ţ', u: 'ú', v: 'ṽ', w: 'ŵ', x: 'ẋ', y: 'ý', z: 'ž',
  A: 'Á', B: 'Ɓ', C: 'Ç', D: 'Ð', E: 'É', F: 'Ƒ', G: 'Ĝ', H: 'Ĥ', I: 'Í', J: 'Ĵ', K: 'Ķ', L: 'Ĺ', M: 'Ṁ',
  N: 'Ñ', O: 'Ó', P: 'Þ', Q: 'Ǫ', R: 'Ŕ', S: 'Š', T: 'Ţ', U: 'Ú', V: 'Ṽ', W: 'Ŵ', X: 'Ẋ', Y: 'Ý', Z: 'Ž',
};

/** How much longer the pseudo text is than the source (German and Finnish often run ~30-40 % longer). */
export const PSEUDO_EXPANSION = 0.4;

/**
 * The pseudo form of `text`: accented letters, padding of at least 40 % of the
 * source length in space-separated groups of up to five `~` (so it can wrap like
 * words), inside `[` `]`. Characters other than ASCII letters (digits,
 * punctuation, bidi isolation marks, emoji) are kept as they are.
 */
export function pseudoLocalize(text: string): string {
  if (text.length === 0) return text;
  let out = '';
  for (const ch of text) out += ACCENTED[ch] ?? ch;
  let pad = Math.ceil(text.length * PSEUDO_EXPANSION);
  const groups: string[] = [];
  while (pad > 0) {
    // A group plus its leading space adds `n + 1` characters.
    const n = Math.max(1, Math.min(5, pad - 1));
    groups.push('~'.repeat(n));
    pad -= n + 1;
  }
  return `[${out} ${groups.join(' ')}]`;
}
