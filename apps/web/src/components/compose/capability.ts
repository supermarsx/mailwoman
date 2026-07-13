// Per-recipient → whole-message transport capability (plan §2.5, e4). Pure logic,
// no Solid/DOM, so the E2EE/TLS/mixed rule is unit-testable in isolation. The
// live banner in `compose-crypto.tsx` renders `computeCapability(...)`.

import type { CryptoKey, KeyKind } from '../../api/crypto-types.ts';

/** The transport the message will actually use, per the frozen banner contract. */
export type TransportCapability = 'e2ee' | 'tls' | 'mixed';

/** One recipient's resolved encryption capability (from `CryptoKey/lookup`). */
export interface RecipientCapability {
  address: string;
  /** True when a usable (non-revoked) PGP key or S/MIME cert is known. */
  encryptable: boolean;
  /** The kind of the chosen usable key, or `null` when none. */
  keyKind: KeyKind | null;
  /** The armored public key / PEM cert to encrypt to, or `null` when none. */
  publicKey: string | null;
}

/**
 * Pick a recipient's best usable key: any key whose TOFU trust is not `revoked`
 * and that carries encryption material (PGP armor or S/MIME PEM). `verified`/
 * `tofu` are preferred over `unverified`, but any non-revoked key is usable for
 * opportunistic encryption (the banner reflects reachability, not trust).
 */
export function chooseRecipientKey(address: string, keys: CryptoKey[]): RecipientCapability {
  const usable = keys.filter((k) => k.trust !== 'revoked' && (k.publicKeyArmored ?? k.certPem) !== null);
  const rank = (k: CryptoKey): number => (k.trust === 'verified' ? 0 : k.trust === 'tofu' ? 1 : 2);
  const best = usable.slice().sort((a, b) => rank(a) - rank(b))[0];
  if (best === undefined) {
    return { address, encryptable: false, keyKind: null, publicKey: null };
  }
  return {
    address,
    encryptable: true,
    keyKind: best.kind,
    publicKey: best.publicKeyArmored ?? best.certPem,
  };
}

/**
 * The frozen banner rule: all recipients encryptable → `e2ee`; none (or no
 * recipients yet) → `tls`; some but not all → `mixed`.
 */
export function computeCapability(recipients: RecipientCapability[]): TransportCapability {
  if (recipients.length === 0) return 'tls';
  const encryptable = recipients.filter((r) => r.encryptable).length;
  if (encryptable === recipients.length) return 'e2ee';
  if (encryptable === 0) return 'tls';
  return 'mixed';
}

/** Normalize a raw recipient list: trim, lowercase, drop blanks, dedupe. */
export function normalizeRecipients(raw: string[]): string[] {
  const seen = new Set<string>();
  for (const r of raw) {
    const addr = r.trim().toLowerCase();
    if (addr.length > 0) seen.add(addr);
  }
  return [...seen];
}
