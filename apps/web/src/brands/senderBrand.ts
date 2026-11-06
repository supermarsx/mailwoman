import type {AuthVerdict} from '../api/security-types.ts';
import {brandForHost, type BrandEntry} from './registry.ts';

/**
 * The brand to show beside a sender, or undefined. A logo is shown only when the From
 * domain belongs to a known brand AND the message passed an aligned DMARC check AND
 * DKIM passed for a domain of that same brand, so a lookalike, a forged From or a
 * message that merely forwards through a brand's servers never gets the logo.
 * Without a check result (message list rows carry none) there is no brand.
 */
export function brandAvatarFor(address: string | null | undefined, auth: AuthVerdict | null | undefined): BrandEntry | undefined {
  if (!address || !auth) return undefined;
  const brand = brandForHost(address);
  if (!brand || brand.noMark) return undefined;
  if (auth.dmarc.result !== 'pass' || auth.dmarc.aligned !== true) return undefined;
  if (auth.dkim.result !== 'pass' || !auth.dkim.domain) return undefined;
  return brandForHost(auth.dkim.domain)?.id === brand.id ? brand : undefined;
}
