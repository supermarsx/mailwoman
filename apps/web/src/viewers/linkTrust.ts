// Trusted link domains (t39 R7). A reviewed message link whose host is on the
// user's (or the administrator's) trusted list opens from the review list
// without the destination/warning step. Matching is on whole DNS labels:
// `example.com` trusts `example.com` only, and with subdomains on also
// `mail.example.com`, never `evilexample.com`, `example.com.evil.test` or an
// IP literal. The Rust side (`settings_registry::link_trust`) applies the same
// rules to stored values; both read `linkTrust.vectors.json`.
import { normalizeTrustedDomain } from '../components/compose/recipientTrust.ts';
import type { MessageLink } from './messageLinks.ts';

export interface LinkTrustPolicy { domains: readonly string[]; subdomains: boolean }

/** A list entry the policy accepts: an exact, lowercase, ASCII (A-label)
 * domain of at least two labels whose last label is not numeric. */
export function normalizeTrustedLinkDomain(value: string): string | null {
  const domain = normalizeTrustedDomain(value);
  if (domain === null) return null;
  const labels = domain.split('.');
  if (labels.length < 2 || /^\d+$/.test(labels[labels.length - 1]!)) return null;
  return domain;
}

/** Whether `hostname` (a URL hostname: lowercase, A-label, no port) is trusted. */
export function trustedLinkHost(hostname: string, policy: LinkTrustPolicy): boolean {
  const host = hostname.toLowerCase().replace(/\.$/, '');
  if (!host || host.startsWith('[') || normalizeTrustedLinkDomain(host) === null) return false;
  for (const raw of policy.domains) {
    const domain = normalizeTrustedLinkDomain(raw);
    if (domain === null) continue;
    if (host === domain) return true;
    if (policy.subdomains && host.endsWith(`.${domain}`)) return true;
  }
  return false;
}

/** A link skips review only when it is HTTPS to a trusted host and carries
 * no warning that is about the destination itself (plain HTTP, IP address,
 * direction-changing characters). A label/destination mismatch does not
 * block trust: the destination, not the label, is what is trusted. */
export function linkBypassesReview(link: MessageLink, policy: LinkTrustPolicy): boolean {
  if (link.kind !== 'web' || policy.domains.length === 0) return false;
  if (link.warnings.some(w => w === 'http' || w === 'address' || w === 'direction')) return false;
  let url: URL; try { url = new URL(link.href); } catch { return false; }
  return url.protocol === 'https:' && trustedLinkHost(url.hostname, policy);
}

/** The trusted-link policy from the settings map (empty until the keys exist). */
export function linkTrustPolicy(values: Readonly<Record<string, { value: unknown } | undefined>>): LinkTrustPolicy {
  const raw = values['links.trustedDomains']?.value;
  const domains = Array.isArray(raw) ? raw.filter((d): d is string => typeof d === 'string') : [];
  return { domains, subdomains: values['links.trustedSubdomains']?.value === true };
}
