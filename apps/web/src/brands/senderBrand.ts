import type {AuthVerdict} from '../api/security-types.ts';
import type {ListAuthSummary} from '../api/jmap-types.ts';
import {brandForHost, type BrandEntry} from './registry.ts';

/**
 * The brand to show beside a sender, or undefined. A logo is shown only when the From
 * domain belongs to a known brand AND the message passed an aligned DMARC check AND
 * DKIM passed for a domain of that same brand, so a lookalike, a forged From or a
 * message that merely forwards through a brand's servers never gets the logo.
 * Without a check result there is no brand. A list row's compact summary
 * (`mailwomanAuth`) is enough: the check reads only DKIM and DMARC.
 */
export function brandAvatarFor(address: string | null | undefined, auth: Pick<AuthVerdict, 'dkim' | 'dmarc'> | ListAuthSummary | null | undefined): BrandEntry | undefined {
  if (!address || !auth) return undefined;
  const brand = brandForHost(address);
  if (!brand || brand.noMark) return undefined;
  if (auth.dmarc.result !== 'pass' || auth.dmarc.aligned !== true) return undefined;
  if (auth.dkim.result !== 'pass' || !auth.dkim.domain) return undefined;
  return brandForHost(auth.dkim.domain)?.id === brand.id ? brand : undefined;
}

/** Government and public-body marks are one generic glyph, not a brand. */
const GENERIC_ICON_ID = 'government';

/**
 * Whether a bundled sender icon (`senderDomainIcon`) may be shown for a message:
 * the same proof as `brandAvatarFor`. A generic glyph needs none. A brand in the
 * logo registry needs `brandAvatarFor` to accept it; a catalogue icon outside the
 * registry needs the same checks with DKIM passing for a domain of that same icon.
 * `lookup` is `senderDomainIcon`, passed in so this module stays free of the
 * icon catalogue.
 */
export function senderIconProven(
  address: string | null | undefined,
  icon: {id: string} | undefined,
  auth: Pick<AuthVerdict, 'dkim' | 'dmarc'> | ListAuthSummary | null | undefined,
  lookup: (address: string) => {id: string} | undefined,
): boolean {
  if (!address || !icon) return false;
  if (icon.id === GENERIC_ICON_ID) return true;
  if (!auth) return false;
  if (brandForHost(address)) return brandAvatarFor(address, auth) !== undefined;
  if (auth.dmarc.result !== 'pass' || auth.dmarc.aligned !== true) return false;
  if (auth.dkim.result !== 'pass' || !auth.dkim.domain) return false;
  return lookup(`x@${auth.dkim.domain}`)?.id === icon.id;
}
