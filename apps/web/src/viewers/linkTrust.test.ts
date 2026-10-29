import { describe, expect, it } from 'vitest';
import vectors from './linkTrust.vectors.json';
import { linkBypassesReview, linkTrustPolicy, normalizeTrustedLinkDomain, trustedLinkHost } from './linkTrust.ts';
import { inspectMessageLink } from './messageLinks.ts';

describe('trusted link domains', () => {
  it.each(vectors.entries)('normalizes $input', ({ input, normalized }) => {
    expect(normalizeTrustedLinkDomain(input)).toBe(normalized);
  });
  it.each(vectors.matches)('$host against $domains (subdomains $subdomains) = $trusted', ({ host, domains, subdomains, trusted }) => {
    expect(trustedLinkHost(host, { domains, subdomains })).toBe(trusted);
  });
  it('bypasses review only for HTTPS links to a trusted host without destination warnings', () => {
    const policy = { domains: ['example.com'], subdomains: true };
    const link = (href: string, label = 'Open') => inspectMessageLink(href, label)!;
    expect(linkBypassesReview(link('https://docs.example.com/a'), policy)).toBe(true);
    expect(linkBypassesReview(link('https://example.com/', 'https://bank.test'), policy)).toBe(true);
    expect(linkBypassesReview(link('http://example.com/'), policy)).toBe(false);
    expect(linkBypassesReview(link('https://example.com.evil.test/'), policy)).toBe(false);
    expect(linkBypassesReview(link('https://evilexample.com/'), policy)).toBe(false);
    expect(linkBypassesReview(link('https://example.com/', 'Open ‮'), policy)).toBe(false);
    expect(linkBypassesReview(link('mailto:a@example.com'), policy)).toBe(false);
    expect(linkBypassesReview(link('https://example.com/'), { domains: [], subdomains: true })).toBe(false);
  });
  it('reads the policy from the settings map and ignores malformed values', () => {
    expect(linkTrustPolicy({})).toEqual({ domains: [], subdomains: false });
    expect(linkTrustPolicy({ 'links.trustedDomains': { value: ['a.test', 3] }, 'links.trustedSubdomains': { value: true } })).toEqual({ domains: ['a.test'], subdomains: true });
    expect(linkTrustPolicy({ 'links.trustedDomains': { value: 'a.test' }, 'links.trustedSubdomains': { value: 'yes' } })).toEqual({ domains: [], subdomains: false });
  });
});
