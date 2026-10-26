import { expect, it } from 'vitest';
import { printMessageDocument } from './printMessage.ts';
import type { Email } from '../api/jmap-types.ts';

const email = { subject: '<script>Subject</script>', from: [{ name: 'Alice <Team>', email: 'alice@example.test' }], to: [{ email: 'bob@example.test', name: null }], receivedAt: '2026-10-25T12:00:00Z' } as Email;
it('escapes message headers and plain text and blocks scripts and external resources', () => {
  const document = printMessageDocument(email, { text: '<img src=https://tracker.example/pixel>' });
  expect(document).toContain('&lt;script&gt;Subject&lt;/script&gt;');
  expect(document).toContain('Alice &lt;Team&gt; &lt;alice@example.test&gt;');
  expect(document).toContain('&lt;img src=https://tracker.example/pixel&gt;');
  expect(document).toContain("default-src 'none'; script-src 'none'; img-src data:");
  expect(document).toContain("base-uri 'none'; form-action 'none'");
  expect(document).not.toContain('<script>');
});
it('retains sanitized rich content under the stricter print resource policy', () => {
  const document = printMessageDocument(email, { html: '<table><tr><td>Invoice</td></tr></table>' });
  expect(document).toContain('<article><table><tr><td>Invoice</td></tr></table></article>');
  expect(document).toContain('color:#000;background:#fff');
  expect(document).toContain('2026-10-25T12:00:00Z');
});
