import {describe, expect, it} from 'vitest';
import {BRANDS, brandForHost} from './registry.ts';
import {brandAvatarFor} from './senderBrand.ts';
import {senderDomainIcon} from '../components/senderIcons/match.ts';

// Differential: the registry matcher (`brandForHost`, reader and list fallback) against the older
// bundled-catalogue matcher (`senderDomainIcon`, list). Both match by registrable-domain suffix at any
// depth, so a subdomain is never the reason an icon is missing. [host, registry id, catalogue id];
// '-' means no match. Where the two disagree on the id the table records it.
const TABLE: [string, string, string][] = [
  ['amazon.com', 'amazon', 'amazon'], ['e.amazon.com', 'amazon', 'amazon'], ['amazon.de', 'amazon', 'amazon'], ['amazon.co.uk', 'amazon', 'amazon'], ['amazon.es', 'amazon', 'amazon'], ['amazon.fr', 'amazon', 'amazon'], ['amazon.it', 'amazon', 'amazon'], ['amazon.com.br', 'amazon', 'amazon'],
  ['zalando.com', 'zalando', 'zalando'], ['mail.zalando.com', 'zalando', 'zalando'], ['zalando.pt', 'zalando', 'zalando'],
  ['paypal.com', 'paypal', 'paypal'], ['em.paypal.com', 'paypal', 'paypal'], ['service.paypal.com', 'paypal', 'paypal'],
  ['github.com', 'github', 'github'], ['notifications.github.com', 'github', 'github'],
  ['vinted.com', 'vinted', 'vinted'], ['news.vinted.com', 'vinted', 'vinted'],
  ['fnac.pt', 'fnac', 'fnac'], ['service.fnac.pt', 'fnac', 'fnac'], ['fnac.com', 'fnac', '-'],
  ['apple.com', 'apple', 'apple'], ['email.apple.com', 'apple', 'apple'], ['id.apple.com', 'apple', 'apple'],
  ['google.com', 'google', 'google'], ['accounts.google.com', 'google', 'google'], ['mail.google.com', 'google', 'google'],
  ['facebookmail.com', 'facebook', 'facebook'], ['facebook.com', 'facebook', 'facebook'],
  ['linkedin.com', 'linkedin', 'linkedin'], ['e.linkedin.com', 'linkedin', 'linkedin'], ['messages-noreply.linkedin.com', 'linkedin', 'linkedin'],
  ['twitter.com', 'x', 'x'], ['x.com', 'x', 'x'], ['netflix.com', 'netflix', 'netflix'], ['mailer.netflix.com', 'netflix', 'netflix'],
  ['spotify.com', 'spotify', 'spotify'], ['mail.spotify.com', 'spotify', 'spotify'],
  ['ebay.com', 'ebay', 'ebay'], ['ebay.co.uk', 'ebay', 'ebay'], ['reply.ebay.com', 'ebay', 'ebay'],
  ['microsoft.com', 'microsoft', 'microsoft'], ['e-mail.microsoft.com', 'microsoft', 'microsoft'], ['account.microsoft.com', 'microsoft', 'microsoft'],
  ['uber.com', 'uber', 'uber'], ['olx.pt', 'olx', 'olx'], ['ctt.pt', 'ctt', 'ctt'], ['mail.ctt.pt', 'ctt', 'ctt'],
  ['millenniumbcp.pt', 'millenniumbcp', 'millenniumbcp'], ['cgd.pt', 'cgd', 'cgd'], ['santander.pt', 'santander', 'santanderpt'], ['mail.santander.pt', 'santander', 'santanderpt'],
  ['novobanco.pt', 'novobanco', 'novobanco'], ['montepio.pt', 'montepio', '-'], ['worten.pt', 'worten', 'worten'], ['mail.worten.pt', 'worten', 'worten'],
  ['continente.pt', 'continente', 'continente'], ['pingodoce.pt', 'pingodoce', 'pingodoce'], ['edp.pt', 'edp', 'edp'],
  ['vodafone.pt', 'vodafone', 'vodafone'], ['meo.pt', 'meo', 'meo'], ['nos.pt', 'nos', 'nos'], ['nowo.pt', 'nowo', 'nowo'],
  ['booking.com', 'bookingdotcom', 'bookingdotcom'], ['mail.booking.com', 'bookingdotcom', 'bookingdotcom'], ['airbnb.com', 'airbnb', 'airbnb'],
  ['ryanair.com', 'ryanair', '-'], ['mail.ryanair.com', 'ryanair', '-'], ['tap.pt', 'tap', '-'], ['flytap.com', 'tap', '-'], ['easyjet.com', 'easyjet', '-'],
  ['dropbox.com', 'dropbox', 'dropbox'], ['mail.dropbox.com', 'dropbox', 'dropbox'], ['slack.com', 'slack', 'slack'], ['mail.slack.com', 'slack', 'slack'],
  ['notion.so', 'notion', 'notion'], ['figma.com', 'figma', 'figma'], ['adobe.com', 'adobe', '-'], ['mail.adobe.com', 'adobe', '-'],
  ['stripe.com', 'stripe', 'stripe'], ['mail.stripe.com', 'stripe', 'stripe'], ['shopify.com', 'shopify', 'shopify'],
  ['mailchimp.com', 'mailchimp', '-'], ['mcsv.net', 'mailchimp', '-'], ['sendgrid.net', 'sendgrid', '-'],
  ['youtube.com', 'youtube', 'youtube'], ['tiktok.com', 'tiktok', 'tiktok'], ['mail.tiktok.com', 'tiktok', 'tiktok'],
  ['instagram.com', 'instagram', 'instagram'], ['mail.instagram.com', 'instagram', 'instagram'], ['pinterest.com', 'pinterest', 'pinterest'],
  ['reddit.com', 'reddit', 'reddit'], ['redditmail.com', 'reddit', 'reddit'], ['discord.com', 'discord', 'discord'],
  ['steampowered.com', 'steam', '-'], ['epicgames.com', 'epicgames', '-'], ['playstation.com', 'playstation', '-'], ['sony.com', 'sony', '-'], ['nintendo.com', 'nintendo', '-'],
  ['ikea.com', 'ikea', '-'], ['hm.com', 'hm', '-'], ['zara.com', 'zara', '-'], ['inditex.com', 'zara', '-'], ['nike.com', 'nike', '-'], ['adidas.com', 'adidas', '-'],
  ['decathlon.pt', 'decathlon', 'decathlon'], ['mail.decathlon.pt', 'decathlon', 'decathlon'], ['leroymerlin.pt', 'leroymerlin', 'leroymerlin'],
  ['ubereats.com', 'ubereats', 'uber'], ['glovoapp.com', 'glovo', 'glovo'], ['bolt.eu', 'bolt', '-'],
  ['ovh.com', 'ovh', 'ovh'], ['hetzner.com', 'hetzner', 'hetzner'], ['digitalocean.com', 'digitalocean', 'digitalocean'],
  ['cloudflare.com', 'cloudflare', 'cloudflare'], ['mail.cloudflare.com', 'cloudflare', 'cloudflare'], ['gitlab.com', 'gitlab', 'gitlab'],
  ['atlassian.com', 'atlassian', 'atlassian'], ['mail.atlassian.com', 'atlassian', 'atlassian'], ['zoom.us', 'zoom', 'zoom'], ['mail.zoom.us', 'zoom', 'zoom'],
  ['docusign.net', 'docusign', '-'], ['docusign.com', 'docusign', '-'], ['wise.com', 'wise', '-'],
  ['revolut.com', 'revolut', 'revolut'], ['mail.revolut.com', 'revolut', 'revolut'], ['n26.com', 'n26', '-'],
  ['klarna.com', 'klarna', 'klarna'], ['mail.klarna.com', 'klarna', 'klarna'], ['mbway.pt', 'mbway', '-'], ['sibs.pt', 'sibs', '-'],
  ['dhl.com', 'dhl', 'dhl'], ['dhl.de', 'dhl', '-'], ['ups.com', 'ups', 'ups'], ['fedex.com', 'fedex', 'fedex'], ['seur.com', 'seur', '-'], ['correos.es', 'correos', '-'],
  ['lemonde.fr', 'lemonde', '-'], ['spiegel.de', 'spiegel', '-'], ['bbc.co.uk', 'bbc', 'bbc'], ['nytimes.com', 'nytimes', '-'], ['mail.nytimes.com', 'nytimes', '-'],
  // Public bodies: the catalogue shows one civic glyph, the registry has a few named agencies.
  ['gov.pt', 'govpt', 'government'], ['portaldasfinancas.gov.pt', 'autoridadetributaria', 'government'], ['sns24.gov.pt', 'sns', 'government'], ['hmrc.gov.uk', '-', 'government'], ['impots.gouv.fr', '-', 'government'], ['ec.europa.eu', '-', 'government'],
  // Lookalikes and unknown hosts: neither matcher may match.
  ['evil.io', '-', '-'], ['paypal.com.evil.io', '-', '-'], ['evilpaypal.com', '-', '-'], ['pay-pal.com', '-', '-'], ['paypal.com-secure.net', '-', '-'], ['notgithub.com', '-', '-'], ['github.com.evil.test', '-', '-'],
];

describe('sender host matching, registry against catalogue', () => {
  it('matches the table of 150+ hosts', () => {
    const bad: string[] = [];
    for (const [host, registry, catalogue] of TABLE) {
      const a = brandForHost(host)?.id ?? '-';
      const b = senderDomainIcon(`x@${host}`)?.id ?? '-';
      if (a !== registry || b !== catalogue) bad.push(`${host}: registry ${a} (want ${registry}), catalogue ${b} (want ${catalogue})`);
    }
    expect(bad).toEqual([]);
    expect(TABLE.length).toBeGreaterThanOrEqual(150);
  });
  it('a subdomain at any depth matches like its parent, with case, a trailing dot and a plus tag', () => {
    for (const [host, registry, catalogue] of TABLE) {
      if (registry === '-' && catalogue === '-') continue;
      for (const sub of ['mail.', 'a.b.c.', 'EM.']) {
        const full = `${sub}${host}`;
        expect(brandForHost(full)?.id ?? '-', full).toBe(registry);
        expect(senderDomainIcon(`x@${full}`)?.id ?? '-', full).toBe(catalogue);
      }
      expect(brandForHost(`user+tag@${host.toUpperCase()}.`)?.id ?? '-').toBe(registry);
    }
  });
  it('every registry host matches its own brand with a subdomain, and never as a prefix or suffix lookalike', () => {
    for (const brand of BRANDS) for (const host of brand.hosts) {
      const own = brandForHost(host);
      expect(own, host).toBeDefined();
      expect(brandForHost(`news.${host}`)?.id, host).toBe(own!.id);
      expect(brandForHost(`${host}.evil.io`)?.id === own!.id, host).toBe(false);
      // `evilaccounts.google.com` is still a subdomain of google.com: only hosts with no registered parent can be glued onto.
      if (!brandForHost(host.split('.').slice(1).join('.'))) expect(brandForHost(`evil${host}`)?.id === own!.id, host).toBe(false);
    }
  });
  it('the decision is the same for a subdomain as for the apex, with or without a verdict', () => {
    const auth = {dkim: {result: 'pass', domain: 'amazon.com'}, dmarc: {result: 'pass', aligned: true}};
    for (const mode of ['never', 'known-failure', 'always'] as const) {
      expect(brandAvatarFor('a@e.amazon.com', auth, mode)?.id).toBe(brandAvatarFor('a@amazon.com', auth, mode)?.id);
    }
    expect(brandAvatarFor('a@e.amazon.com', null)?.id).toBe('amazon');
  });
});
