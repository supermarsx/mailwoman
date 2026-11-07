import type {AuthVerdict} from '../api/security-types.ts';
import type {ListAuthSummary} from '../api/jmap-types.ts';
import {brandForHost, type BrandEntry} from './registry.ts';

/** `mail.brandAvatars.requireAuth`: how much proof a sender brand icon needs. */
export type RequireAuth = 'never' | 'known-failure' | 'always';
export const REQUIRE_AUTH_MODES: readonly RequireAuth[] = ['never', 'known-failure', 'always'];
export const DEFAULT_REQUIRE_AUTH: RequireAuth = 'known-failure';
export const isRequireAuth = (value: unknown): value is RequireAuth => typeof value === 'string' && (REQUIRE_AUTH_MODES as readonly string[]).includes(value);

/** What a list row (`ListAuthSummary`) or the reader's full verdict gives us; SPF is only in the full verdict. */
export type AuthEvidence = Pick<AuthVerdict, 'dkim' | 'dmarc'> & {spf?: {result: string}} | ListAuthSummary;

/**
 * Whether the cached verdict says the message FAILED authentication: DMARC failed, or DKIM
 * failed and DMARC did not pass (so no aligned SPF rescued it), or SPF and DKIM both failed.
 * No verdict, or a verdict with only "none"/"neutral"/temporary results, is unknown, not failed.
 */
export function authFailed(auth: AuthEvidence | null | undefined): boolean {
  if (!auth) return false;
  if (auth.dmarc.result === 'fail') return true;
  if (auth.dkim.result === 'fail' && auth.dmarc.result !== 'pass') return true;
  return auth.dkim.result === 'fail' && 'spf' in auth && auth.spf?.result === 'fail';
}

/** The strict proof of `always` mode: aligned DMARC pass plus DKIM pass for a domain of the same brand. */
function authProves(brandId: string, auth: AuthEvidence | null | undefined): boolean {
  if (!auth) return false;
  if (auth.dmarc.result !== 'pass' || auth.dmarc.aligned !== true) return false;
  if (auth.dkim.result !== 'pass' || !auth.dkim.domain) return false;
  return brandForHost(auth.dkim.domain)?.id === brandId;
}

export type IconDecision = 'show' | 'unverified' | 'hide';

/** Decide whether a matched icon may be shown for a message in the given mode. */
export function authDecision(mode: RequireAuth, proves: () => boolean, auth: AuthEvidence | null | undefined): IconDecision {
  if (mode === 'never') return 'show';
  if (mode === 'always') return proves() ? 'show' : 'unverified';
  return authFailed(auth) ? 'unverified' : 'show';
}

/**
 * The brand to show beside a sender. The From domain (and any subdomain of it) must belong to
 * a known brand. `mode` then decides what the message's authentication changes: `never` ignores
 * it, `known-failure` (default) hides the logo only when the cached verdict says the message
 * failed (a message not yet checked keeps its logo), `always` also needs an aligned DMARC pass
 * with DKIM for a domain of the same brand. Public mailbox providers (Gmail, Outlook) are
 * never a sender brand: their addresses belong to people.
 */
export function brandAvatarDecision(address: string | null | undefined, auth: AuthEvidence | null | undefined, mode: RequireAuth = DEFAULT_REQUIRE_AUTH): {brand: BrandEntry | undefined; unverified: boolean} {
  const brand = address ? brandForHost(address) : undefined;
  if (!brand || brand.noMark || brand.category === 'mail') return {brand: undefined, unverified: false};
  const decision = authDecision(mode, () => authProves(brand.id, auth), auth);
  return decision === 'show' ? {brand, unverified: false} : {brand: undefined, unverified: true};
}

export function brandAvatarFor(address: string | null | undefined, auth: AuthEvidence | null | undefined, mode: RequireAuth = DEFAULT_REQUIRE_AUTH): BrandEntry | undefined {
  return brandAvatarDecision(address, auth, mode).brand;
}

/** Government and public-body marks are one generic glyph, not a brand. */
const GENERIC_ICON_ID = 'government';

/**
 * Whether a bundled catalogue icon (`senderDomainIcon`) may be shown for a message. A generic
 * glyph needs no proof. Otherwise the mode decides as in `brandAvatarDecision`; `always` proves a
 * registry brand with `brandAvatarFor`, and a catalogue icon outside the registry with DKIM
 * passing for a domain of that same icon. `lookup` is `senderDomainIcon`, passed in so this
 * module stays free of the icon catalogue.
 */
export function senderIconDecision(
  address: string | null | undefined,
  icon: {id: string} | undefined,
  auth: AuthEvidence | null | undefined,
  lookup: (address: string) => {id: string} | undefined,
  mode: RequireAuth = DEFAULT_REQUIRE_AUTH,
): IconDecision {
  if (!address || !icon) return 'hide';
  if (icon.id === GENERIC_ICON_ID) return 'show';
  const proves = (): boolean => {
    if (!auth || auth.dmarc.result !== 'pass' || auth.dmarc.aligned !== true || auth.dkim.result !== 'pass' || !auth.dkim.domain) return false;
    const registry = brandForHost(address);
    if (registry) return brandForHost(auth.dkim.domain)?.id === registry.id;
    return lookup(`x@${auth.dkim.domain}`)?.id === icon.id;
  };
  return authDecision(mode, proves, auth);
}
