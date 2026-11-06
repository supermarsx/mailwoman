// Which identity a reply, reply-all or forward is sent from (mail.replyFromMode).
//
// match (default) picks the identity the original was addressed to. Where the
// message names its recipient is configurable (mail.replyFrom.headerPriority):
// by default Delivered-To, then X-Original-To, then To, then Cc. Within one
// header the first kind of match that hits wins, in this order:
//   1. the exact address (case-insensitive),
//   2. plus-addressing: user+tag@host reaches user@host (either side),
//   3. a wildcard identity address: *@host (catch-all), user+*@host,
//      user@*.host (any subdomain),
//   4. (delivery headers only, not To or Cc) the same domain, or a parent
//      domain of the received address: sales@mail.example.org, delivered
//      to a catch-all, reaches the identity at example.org.
// default always uses the default identity, keep leaves the composer's
// identity alone, and ask matches like match but reports every candidate so
// the composer can show a small chooser. No match falls back to the default.

export type ReplyFromMode = 'match' | 'default' | 'ask' | 'keep';
export const REPLY_FROM_MODES: readonly ReplyFromMode[] = ['match', 'default', 'ask', 'keep'];
export const isReplyFromMode = (value: unknown): value is ReplyFromMode => typeof value === 'string' && (REPLY_FROM_MODES as readonly string[]).includes(value);

/** A header that can name the address a message reached us at. */
export type HeaderSource = 'delivered-to' | 'x-original-to' | 'envelope-to' | 'x-forwarded-to' | 'resent-to' | 'to' | 'cc';
export const HEADER_SOURCES: readonly HeaderSource[] = ['delivered-to', 'x-original-to', 'envelope-to', 'x-forwarded-to', 'resent-to', 'to', 'cc'];
/** The header names as written in the setting and shown to the user. */
export const HEADER_LABELS: Readonly<Record<HeaderSource, string>> = {
  'delivered-to': 'Delivered-To',
  'x-original-to': 'X-Original-To',
  'envelope-to': 'Envelope-To',
  'x-forwarded-to': 'X-Forwarded-To',
  'resent-to': 'Resent-To',
  to: 'To',
  cc: 'Cc',
};
export const DEFAULT_HEADER_PRIORITY: readonly HeaderSource[] = ['delivered-to', 'x-original-to', 'to', 'cc'];

/** The setting's list, read leniently: unknown names are dropped, repeats ignored, case ignored. */
export function parseHeaderPriority(value: unknown): HeaderSource[] {
  if (!Array.isArray(value)) return [...DEFAULT_HEADER_PRIORITY];
  const out: HeaderSource[] = [];
  for (const item of value) {
    if (typeof item !== 'string') continue;
    const key = item.trim().toLowerCase();
    const found = HEADER_SOURCES.find((source) => source === key);
    if (found !== undefined && !out.includes(found)) out.push(found);
  }
  return out.length > 0 ? out : [...DEFAULT_HEADER_PRIORITY];
}

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
  /** Delivered-To, kept for callers that only know that one header. */
  delivered?: readonly string[];
  /** The addresses of each delivery header, by source. */
  headers?: Partial<Record<HeaderSource, readonly string[]>>;
}

export type ReplyIdentityReason = 'match' | 'match-plus' | 'match-pattern' | 'match-domain' | 'ambiguous' | 'default' | 'forced-default' | 'keep';

export interface ReplyIdentityChoice {
  /** The identity to select; null means leave the composer's current choice. */
  identityId: string | null;
  reason: ReplyIdentityReason;
  /** The received address that matched (lowercased). */
  address?: string;
  /** The header that carried it. */
  source?: HeaderSource;
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

const escapeRegExp = (text: string): string => text.replace(/[.+?^${}()|[\]\\]/g, '\\$&');

/** An identity address that uses `*` (any run of characters inside the local part or the domain). */
export const isAddressPattern = (address: string): boolean => address.includes('*');

/**
 * Whether `pattern` (lowercase, with `*`) matches `address` (lowercase). `*`
 * never crosses the `@`; `*@host` is a catch-all, `user+*@host` any tag, and
 * `user@*.host` any subdomain (one or more labels).
 */
export function matchesPattern(pattern: string, address: string): boolean {
  const at = pattern.lastIndexOf('@');
  const target = address.lastIndexOf('@');
  if (at <= 0 || target <= 0 || !pattern.includes('*')) return false;
  const part = (glob: string, subdomain: boolean): string =>
    glob
      .split('*')
      .map(escapeRegExp)
      .join(subdomain ? '[^@]+' : '[^@]*');
  const domain = pattern.slice(at + 1);
  const re = new RegExp(`^${part(pattern.slice(0, at), false)}@${part(domain, domain.startsWith('*.'))}$`);
  return re.test(address);
}

/** The domain of an address, and each parent domain below the top two labels. */
function domainChain(address: string): string[] {
  const at = address.lastIndexOf('@');
  if (at < 0) return [];
  const labels = address.slice(at + 1).split('.').filter(Boolean);
  const chain: string[] = [];
  for (let i = 0; i < labels.length - 1; i += 1) chain.push(labels.slice(i).join('.'));
  return chain;
}

type Tier = 'exact' | 'plus' | 'pattern' | 'domain';
const TIERS: readonly Tier[] = ['exact', 'plus', 'pattern', 'domain'];

function tierMatches(tier: Tier, own: string, address: string): boolean {
  switch (tier) {
    case 'exact': return own === address;
    case 'plus': return own !== address && !isAddressPattern(own) && stripPlusTag(own) === stripPlusTag(address);
    case 'pattern': return isAddressPattern(own) && matchesPattern(own, address);
    case 'domain': {
      if (isAddressPattern(own)) return false;
      const ownDomain = own.slice(own.lastIndexOf('@') + 1);
      return ownDomain !== '' && domainChain(address).includes(ownDomain);
    }
  }
}

const REASON_OF: Record<Tier, ReplyIdentityReason> = { exact: 'match', plus: 'match-plus', pattern: 'match-pattern', domain: 'match-domain' };

function addressesFor(source: HeaderSource, received: ReceivedAddresses): readonly string[] {
  if (source === 'to') return received.to;
  if (source === 'cc') return received.cc ?? [];
  const named = received.headers?.[source];
  if (named !== undefined) return named;
  return source === 'delivered-to' ? (received.delivered ?? []) : [];
}

export function chooseReplyIdentity(
  identities: readonly FromCandidate[],
  received: ReceivedAddresses,
  mode: ReplyFromMode,
  priority: readonly HeaderSource[] = DEFAULT_HEADER_PRIORITY,
): ReplyIdentityChoice {
  if (mode === 'keep') return { identityId: null, reason: 'keep', alternatives: [] };
  const fallback = defaultIdentity(identities);
  const asDefault = (reason: ReplyIdentityReason): ReplyIdentityChoice => ({ identityId: fallback?.id ?? null, reason, alternatives: [] });
  if (mode === 'default') return asDefault('forced-default');

  for (const source of priority) {
    const addresses = addressesFor(source, received).map(bareAddress).filter((value) => value !== '');
    // To and Cc list other people too, so a shared domain says nothing there.
    const tiers = source === 'to' || source === 'cc' ? TIERS.filter((tier) => tier !== 'domain') : TIERS;
    for (const tier of tiers) {
      const hits: { identity: FromCandidate; address: string }[] = [];
      for (const address of addresses) {
        for (const identity of identities) {
          if (tierMatches(tier, bareAddress(identity.email), address)) hits.push({ identity, address });
        }
      }
      if (hits.length === 0) continue;
      const unique = [...new Map(hits.map((hit) => [hit.identity.id, hit])).values()];
      const first = unique[0]!;
      const reason = REASON_OF[tier];
      if (unique.length === 1) return { identityId: first.identity.id, reason, address: first.address, source, alternatives: [] };
      // Several: the variant whose address equals the received one, else the default among them, else the first.
      const candidates = unique.map((hit) => hit.identity);
      const exactVariants = unique.filter((hit) => hit.identity.variantOf && bareAddress(hit.identity.email) === hit.address);
      const pick = exactVariants.length === 1 ? exactVariants[0]!.identity : (candidates.find((item) => item.isDefault) ?? defaultIdentity(candidates)!);
      return {
        identityId: pick.id,
        reason: mode === 'ask' ? 'ambiguous' : reason,
        address: first.address,
        source,
        alternatives: candidates.filter((item) => item.id !== pick.id),
      };
    }
  }
  return asDefault('default');
}
