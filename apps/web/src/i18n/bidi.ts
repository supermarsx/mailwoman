// Unicode bidi-isolation helpers (SPEC §24 — mixed-direction subject spoofing).
//
// An attacker can craft a subject/display-name that, interpolated into a UI
// string, visually reorders surrounding text (the classic RTL-override spoof:
// "‮gnp.exe" rendering as "exe.png"). When a user-controlled string is
// dropped into a translated message, wrap it in a First-Strong Isolate so its
// bidi run cannot leak into the neighbouring UI text.
//
// Fluent's own placeable isolation (`useIsolating`) is turned OFF in this runtime
// (it would inject FSI/PDI around every `{ $var }`, which breaks literal-text
// test assertions). Instead, callers isolate the SPECIFIC untrusted values —
// subjects, display names, filenames — with `isolate()` before passing them as
// `t()` args, keeping isolation surgical and test output clean.

/** First-Strong Isolate — opens a run whose base direction is auto-detected. */
const FSI = '⁨';
/** Pop Directional Isolate — closes the nearest open isolate. */
const PDI = '⁩';
/** Left-to-Right / Right-to-Left marks, for forcing a base direction. */
const LRM = '‎';
const RLM = '‏';

/**
 * Wrap untrusted text (subject, display name, filename) in a bidi isolate so its
 * direction cannot reorder the surrounding UI. Auto-detects the run's direction
 * (FSI). Safe to pass any string; `null`/`undefined` collapse to `''`.
 */
export function isolate(text: string | null | undefined): string {
  const s = text ?? '';
  if (s === '') return '';
  return `${FSI}${stripIsolates(s)}${PDI}`;
}

/**
 * Force a base direction on a run (rarely needed vs. `isolate`, but handy for a
 * value known to be a specific direction, e.g. a bare LTR id shown inside RTL UI).
 */
export function isolateDir(text: string | null | undefined, dir: 'ltr' | 'rtl'): string {
  const s = text ?? '';
  if (s === '') return '';
  const mark = dir === 'rtl' ? RLM : LRM;
  return `${FSI}${mark}${stripIsolates(s)}${PDI}`;
}

/** Remove any pre-existing isolate/override control chars an attacker embedded. */
export function stripIsolates(s: string): string {
  // FSI, LRI, RLI, PDI, LRO, RLO, PDF, LRE, RLE.
  return s.replace(/[⁦-⁩‪-‮]/g, '');
}
