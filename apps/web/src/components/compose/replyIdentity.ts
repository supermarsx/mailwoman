// Which identity a reply, reply-all or forward is sent from (mail.replyFromMode).
//
// match (default) picks the identity the original was addressed to, looking at
// To, then Cc, then any Delivered-To / X-Original-To style header. Matching is
// case-insensitive and understands plus-addressing (user+tag@host reaches
// user@host). Exact address matches beat plus-tag matches. default always uses
// the default identity, keep leaves the composer's identity alone, and ask
// matches like match but reports every candidate so the composer can show a
// small chooser.

export type ReplyFromMode = 'match' | 'default' | 'ask' | 'keep';
export const REPLY_FROM_MODES: readonly ReplyFromMode[] = ['match', 'default', 'ask', 'keep'];
export const isReplyFromMode = (value: unknown): value is ReplyFromMode => typeof value === 'string' && (REPLY_FROM_MODES as readonly string[]).includes(value);

export interface FromCandidate {
  id: string;
  email: string;
  isDefault?: boolean;
  /** Set on a variant: the id of the base identity it belongs to. */
  variantOf?: string;
}

/** Addresses the original message was delivered to, by header. */
export interface ReceivedAddresses {
  to: readonly string[];
  cc?: readonly string[];
  delivered?: readonly string[];
}

export type ReplyIdentityReason = 'match' | 'match-plus' | 'ambiguous' | 'default' | 'forced-default' | 'keep';

export interface ReplyIdentityChoice {
  /** The identity to select; null means leave the composer's current choice. */
  identityId: string | null;
  reason: ReplyIdentityReason;
  /** The received address that matched (lowercased). */
  address?: string;
  /** Other equally good identities, for the ask chooser. */
  alternatives: FromCandidate[];
}

/** Pull the bare address out of `Name <a@b>` or `a@b`; lowercased. */
export function bareAddress(value: string): string {
  const angle = /<([^<>]+)>\s*$/.exec(value);
  return (angle ? angle[1]! : value).trim().toLowerCase();
}

/** `user+tag@host` becomes `user@host`; other addresses are unchanged. */
export function stripPlusTag(address: string): string {
  const at = address.lastIndexOf('@');
  if (at <= 0) return address;
  const local = address.slice(0, at);
  const plus = local.indexOf('+');
  return plus > 0 ? local.slice(0, plus) + address.slice(at) : address;
}

export function defaultIdentity<T extends FromCandidate>(identities: readonly T[]): T | undefined {
  return identities.find((item) => item.isDefault && !item.variantOf) ?? identities.find((item) => item.isDefault) ?? identities.find((item) => !item.variantOf) ?? identities[0];
}

export function chooseReplyIdentity(identities: readonly FromCandidate[], received: ReceivedAddresses, mode: ReplyFromMode): ReplyIdentityChoice {
  if (mode === 'keep') return { identityId: null, reason: 'keep', alternatives: [] };
  const fallback = defaultIdentity(identities);
  const asDefault = (reason: ReplyIdentityReason): ReplyIdentityChoice => ({ identityId: fallback?.id ?? null, reason, alternatives: [] });
  if (mode === 'default') return asDefault('forced-default');

  for (const group of [received.to, received.cc ?? [], received.delivered ?? []]) {
    const addresses = group.map(bareAddress).filter((value) => value !== '');
    // Exact (case-insensitive) first, then plus-tag stripped on both sides.
    for (const plus of [false, true]) {
      const hits: { identity: FromCandidate; address: string }[] = [];
      for (const address of addresses) {
        for (const identity of identities) {
          const own = bareAddress(identity.email);
          const matches = plus ? own !== address && stripPlusTag(own) === stripPlusTag(address) : own === address;
          if (matches) hits.push({ identity, address });
        }
      }
      if (hits.length === 0) continue;
      const unique = [...new Map(hits.map((hit) => [hit.identity.id, hit])).values()];
      const first = unique[0]!;
      if (unique.length === 1) return { identityId: first.identity.id, reason: plus ? 'match-plus' : 'match', address: first.address, alternatives: [] };
      // Several: the variant whose address equals the received one, else the default among them, else the first.
      const candidates = unique.map((hit) => hit.identity);
      const exactVariants = unique.filter((hit) => hit.identity.variantOf && bareAddress(hit.identity.email) === hit.address);
      const pick = exactVariants.length === 1 ? exactVariants[0]!.identity : (candidates.find((item) => item.isDefault) ?? defaultIdentity(candidates)!);
      return {
        identityId: pick.id,
        reason: mode === 'ask' ? 'ambiguous' : plus ? 'match-plus' : 'match',
        address: first.address,
        alternatives: candidates.filter((item) => item.id !== pick.id),
      };
    }
  }
  return asDefault('default');
}
