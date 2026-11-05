// Memory-only cache of sanitised HTML, keyed by a content hash and bounded by bytes.
//
// Re-renders, hot-patching, mode toggles, prefetch-then-open and refetches hand the
// sanitiser the same body again; the answer depends only on (input, options, sanitiser
// version), so it is served from here. Nothing is persisted; the owner clears it on
// sign-out and account switch (`index.ts`). The key is two independent 53-bit hashes
// plus the input length, so a collision would need to match all three.

/** Bump when the sanitiser output for an unchanged input changes. */
export const SANITIZER_VERSION = 2;

/** cyrb53 with a seed: a fast, well-distributed 53-bit string hash. */
function cyrb53(s: string, seed: number): number {
  let h1 = 0xdeadbeef ^ seed;
  let h2 = 0x41c6ce57 ^ seed;
  for (let i = 0; i < s.length; i += 1) {
    const ch = s.charCodeAt(i);
    h1 = Math.imul(h1 ^ ch, 2654435761);
    h2 = Math.imul(h2 ^ ch, 1597334677);
  }
  h1 = Math.imul(h1 ^ (h1 >>> 16), 2246822507) ^ Math.imul(h2 ^ (h2 >>> 13), 3266489909);
  h2 = Math.imul(h2 ^ (h2 >>> 16), 2246822507) ^ Math.imul(h1 ^ (h1 >>> 13), 3266489909);
  return 4294967296 * (2097151 & h2) + (h1 >>> 0);
}

/** `options` identifies the operation and flags; it is part of the key. */
export function cacheKey(options: string, html: string): string {
  return `${SANITIZER_VERSION}|${options}|${html.length}|${cyrb53(html, 0)}|${cyrb53(html, 0x9e3779b9)}`;
}

export class SanitizeCache {
  private readonly entries = new Map<string, string>();
  private bytes = 0;

  /** `maxBytes` bounds the held output (UTF-16, so two bytes per code unit). */
  constructor(private maxBytes: number) {}

  setLimit(maxBytes: number): void {
    this.maxBytes = maxBytes;
    this.evict();
  }

  get(key: string): string | undefined {
    const hit = this.entries.get(key);
    if (hit === undefined) return undefined;
    this.entries.delete(key);
    this.entries.set(key, hit); // most recently used goes last
    return hit;
  }

  set(key: string, value: string): void {
    const size = value.length * 2;
    if (size > this.maxBytes / 2) return; // one body must not flush the cache
    const old = this.entries.get(key);
    if (old !== undefined) {
      this.bytes -= old.length * 2;
      this.entries.delete(key);
    }
    this.entries.set(key, value);
    this.bytes += size;
    this.evict();
  }

  clear(): void {
    this.entries.clear();
    this.bytes = 0;
  }

  get size(): number { return this.entries.size; }
  get byteSize(): number { return this.bytes; }

  private evict(): void {
    for (const [key, value] of this.entries) {
      if (this.bytes <= this.maxBytes) return;
      this.entries.delete(key);
      this.bytes -= value.length * 2;
    }
  }
}
