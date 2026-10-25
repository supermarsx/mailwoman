/** Compact plain-text target for transient UI feedback; never used for logging. */
export function toastTarget(value: string | null | undefined, fallback: string): string {
  const clean = Array.from(value ?? '', char => {
    const code = char.codePointAt(0)!;
    return code < 32 || (code >= 127 && code <= 159) || (code >= 0x202a && code <= 0x202e) || (code >= 0x2066 && code <= 0x2069) ? ' ' : char;
  }).join('').replace(/\s+/g, ' ').trim();
  const points = Array.from(clean || fallback);
  return points.length > 80 ? `${points.slice(0, 79).join('')}…` : points.join('');
}
